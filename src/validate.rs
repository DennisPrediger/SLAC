use crate::{
    ast::Expression,
    environment::{FunctionResult, ValidateEnvironment},
    error::{Error, Result},
    operator::Operator,
    value::Value,
};

/// Validates [`Variable`](Expression::Variable) and [`Call`](Expression::Call)
/// [`Expressions`](Expression) by walking the tree.
/// # Errors
/// Returns an [`Error`] on missing Variables or Functions.
pub fn check_variables_and_functions(
    env: &dyn ValidateEnvironment,
    expression: &Expression,
) -> Result<()> {
    match expression {
        Expression::Unary { right, operator: _ } => check_variables_and_functions(env, right),
        Expression::Binary {
            left,
            right,
            operator: _,
        } => check_variables_and_functions(env, left)
            .and_then(|_| check_variables_and_functions(env, right)),
        Expression::Array {
            expressions: values,
        } => validate_expr_vec(env, values),
        Expression::Variable { name } => {
            if env.variable_exists(name) {
                Ok(())
            } else {
                Err(Error::MissingVariable(name.clone()))
            }
        }
        Expression::Call { name, params } => match env.function_exists(name, params.len()) {
            FunctionResult::Exists => validate_expr_vec(env, params),
            FunctionResult::NotFound => Err(Error::MissingFunction(name.clone())),
            FunctionResult::WrongArity(found, expected) => {
                Err(Error::ParamCountMismatch(name.clone(), found, expected))
            }
        },
        Expression::Literal { value: _ } => Ok(()),
    }
}

fn validate_expr_vec(env: &dyn ValidateEnvironment, expressions: &[Expression]) -> Result<()> {
    expressions
        .iter()
        .try_for_each(|expression| check_variables_and_functions(env, expression))
}

/// Checks if the top level [`Expression`] produces a [`Value::Boolean`] result.
///
/// # Examples
/// ```
/// use slac::{check_boolean_result, Expression, Operator, Value};
///
/// let ast = Expression::Binary {
///     left: Box::new(Expression::Literal{value: Value::Boolean(true)}),
///     right: Box::new(Expression::Literal{value: Value::Boolean(true)}),
///     operator: Operator::And,
/// };
///
/// assert!(check_boolean_result(&ast).is_ok());
/// ```
/// # Errors
///
/// Returns an [`Error`] when the top most Expression can't evaluate to a [`Value::Boolean`].
pub fn check_boolean_result(ast: &Expression) -> Result<()> {
    match ast {
        Expression::Unary { right: _, operator } => match operator {
            Operator::Not => Ok(()),
            _ => Err(Error::InvalidUnaryOperator(*operator)),
        },
        Expression::Binary {
            left: _,
            right: _,
            operator,
        } => match operator {
            Operator::Greater
            | Operator::GreaterEqual
            | Operator::Less
            | Operator::LessEqual
            | Operator::Equal
            | Operator::NotEqual
            | Operator::And
            | Operator::Or => Ok(()),
            _ => Err(Error::InvalidBinaryOperator(*operator)),
        },
        Expression::Array { expressions: _ } => Err(Error::LiteralNotBoolean),
        Expression::Literal { value } => match value {
            Value::Boolean(_) => Ok(()),
            _ => Err(Error::LiteralNotBoolean),
        },
        Expression::Variable { name: _ } | Expression::Call { name: _, params: _ } => {
            Ok(()) // Type not known
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression,
        environment::{Arity, StaticEnvironment},
        operator::Operator,
        stdlib::NativeResult,
        validate::Error,
        value::Value,
    };

    use super::check_variables_and_functions;

    #[test]
    fn valid() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            operator: Operator::Plus,
        };

        let result = check_variables_and_functions(&StaticEnvironment::default(), &ast);

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn valid_nested() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Unary {
                right: Box::new(Expression::Literal {
                    value: Value::Number(10.0),
                }),
                operator: Operator::Minus,
            }),
            operator: Operator::Plus,
        };

        let result = check_variables_and_functions(&StaticEnvironment::default(), &ast);

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn err_missing_variable() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Variable {
                name: String::from("VAR_NAME"),
            }),
            operator: Operator::Plus,
        };

        let result = check_variables_and_functions(&StaticEnvironment::default(), &ast);

        assert_eq!(
            Err(Error::MissingVariable(String::from("VAR_NAME"))),
            result
        );
    }

    #[test]
    fn err_function_missing() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Call {
                name: String::from("max"),
                params: vec![],
            }),
            operator: Operator::Plus,
        };

        let result = check_variables_and_functions(&StaticEnvironment::default(), &ast);

        assert_eq!(Err(Error::MissingFunction(String::from("max"))), result);
    }

    fn dummy_function(_params: &[Value]) -> NativeResult {
        Ok(Value::Boolean(true))
    }

    #[test]
    fn err_function_params_mismatch() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Call {
                name: String::from("max"),
                params: vec![],
            }),
            operator: Operator::Plus,
        };

        let mut env = StaticEnvironment::default();
        env.add_function(
            "max",
            dummy_function,
            Arity::Polyadic {
                required: 2,
                optional: 0,
            },
        );

        let result = check_variables_and_functions(&env, &ast);

        assert_eq!(
            Err(Error::ParamCountMismatch(String::from("max"), 0, 2)),
            result
        );
    }

    #[test]
    fn err_function_nested_params() {
        let ast = Expression::Call {
            name: String::from("func"),
            params: vec![Expression::Variable {
                name: String::from("not_found"),
            }],
        };

        let mut env = StaticEnvironment::default();
        env.add_function("func", dummy_function, Arity::Variadic);

        let result = check_variables_and_functions(&env, &ast);

        assert_eq!(
            Err(Error::MissingVariable(String::from("not_found"))),
            result
        );
    }
}
