use crate::{
    ast::Expression,
    environment::{Environment, FunctionResult},
    error::{Error, Result},
    operator::Operator,
    value::Value,
};

/// Validates [`Variable`](Expression::Variable) and [`Call`](Expression::Call) [`Expressions`](Expression)
/// by walking the AST and returning the first error.
///
/// # Errors
///
/// Returns an [`Error`] on missing variables or functions.
pub fn check_variables_and_functions(
    env: &impl Environment,
    expression: &Expression,
) -> Result<()> {
    match expression {
        Expression::Unary { right, operator: _ } => check_variables_and_functions(env, right),
        Expression::Binary {
            left,
            right,
            operator: _,
        } => check_variables_and_functions(env, left)
            .and_then(|()| check_variables_and_functions(env, right)),
        Expression::Ternary {
            left,
            middle,
            right,
            operator: _,
        } => check_variables_and_functions(env, left)
            .and_then(|()| check_variables_and_functions(env, middle))
            .and_then(|()| check_variables_and_functions(env, right)),
        Expression::Array {
            expressions: values,
        } => check_expressions(env, values),
        Expression::Variable { name } => {
            if env.variable_exists(name) {
                Ok(())
            } else {
                Err(Error::MissingVariable(name.clone()))
            }
        }
        Expression::Call { name, params } => {
            let param_count = params.len();

            match env.function_exists(name, param_count) {
                FunctionResult::Exists { pure: _ } => check_expressions(env, params),
                FunctionResult::NotFound => Err(Error::MissingFunction(name.clone())),
                FunctionResult::WrongArity { min, max } => Err(Error::ParamCountMismatch(
                    name.clone(),
                    param_count,
                    min,
                    max,
                )),
            }
        }
        Expression::Literal { value: _ } => Ok(()),
    }
}

fn check_expressions(env: &impl Environment, expressions: &[Expression]) -> Result<()> {
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
        Expression::Ternary {
            left,
            middle,
            right,
            operator,
        } => match operator {
            Operator::TernaryCondition => {
                // the `left` argument should be a boolean for the `TernaryCondition` to function
                // the `middle` and `right` arguments eventually result in the expressions final result
                check_boolean_result(left)
                    .and_then(|()| check_boolean_result(middle))
                    .and_then(|()| check_boolean_result(right))
            }
            _ => Err(Error::InvalidTernaryOperator(*operator)),
        },
        Expression::Array { expressions: _ } => Err(Error::LiteralNotBoolean),
        Expression::Literal { value } => match value {
            Value::Boolean(_) => Ok(()),
            _ => Err(Error::LiteralNotBoolean),
        },
        Expression::Variable { name: _ } | Expression::Call { name: _, params: _ } => {
            Ok(()) // the type is not known
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression,
        environment::StaticEnvironment,
        function::{Arity, Function},
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
        unreachable!()
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
        env.add_function(Function::new(
            dummy_function,
            Arity::Polyadic {
                required: 2,
                optional: 0,
            },
            "max(left: Number, right: Number): Number",
        ));

        let result = check_variables_and_functions(&env, &ast);

        assert_eq!(
            Err(Error::ParamCountMismatch(String::from("max"), 0, 2, 2)),
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
        env.add_function(Function::new(
            dummy_function,
            Arity::Variadic,
            "func(...): Number",
        ));

        let result = check_variables_and_functions(&env, &ast);

        assert_eq!(
            Err(Error::MissingVariable(String::from("not_found"))),
            result
        );
    }
}
