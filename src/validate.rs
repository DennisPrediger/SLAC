use crate::{
    ast::Expression,
    environment::{FunctionResult, ValidateEnvironment},
    error::{Error, Result},
    operator::Operator,
    value::Value,
};

/// Validates `Variable` and `Call` [`Expressions`](crate::ast::Expression) by walking
/// the tree and returning a [`SlacError`] on the first error.
pub fn validate_env(env: &dyn ValidateEnvironment, expression: &Expression) -> Result<()> {
    match expression {
        Expression::Unary { right, operator: _ } => validate_env(env, right),
        Expression::Binary {
            left,
            right,
            operator: _,
        } => validate_env(env, left).and_then(|_| validate_env(env, right)),
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
            FunctionResult::WrongArity(expected) => Err(Error::ParamCountMismatch(
                name.clone(),
                params.len(),
                expected,
            )),
        },
        Expression::Literal { value: _ } => Ok(()),
    }
}

fn validate_expr_vec(env: &dyn ValidateEnvironment, expressions: &[Expression]) -> Result<()> {
    expressions
        .iter()
        .try_for_each(|expression| validate_env(env, expression))
}

/// Checks if the top level [`Expression`] produces a [`Value::Boolean`] result.
///
/// # Examples
///
/// ```
/// use slac::validate::{validate_boolean_result};
/// use slac::ast::Expression;
/// use slac::value::Value;
/// use slac::operator::Operator;
///
/// let ast = Expression::Binary {
///     left: Box::new(Expression::Literal{value: Value::Boolean(true)}),
///     right: Box::new(Expression::Literal{value: Value::Boolean(true)}),
///     operator: Operator::And,
/// };
///
/// assert!(validate_boolean_result(&ast).is_ok());
/// ```
pub fn validate_boolean_result(ast: &Expression) -> Result<()> {
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
        ast::Expression, environment::StaticEnvironment, operator::Operator, validate::Error,
        value::Value,
    };

    use super::validate_env;

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

        let result = validate_env(&StaticEnvironment::default(), &ast);

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

        let result = validate_env(&StaticEnvironment::default(), &ast);

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn err_missing_variable() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Variable {
                name: "VAR_NAME".to_string(),
            }),
            operator: Operator::Plus,
        };

        let result = validate_env(&StaticEnvironment::default(), &ast);

        assert_eq!(Err(Error::MissingVariable("VAR_NAME".to_string())), result);
    }

    #[test]
    fn err_function_missing() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Call {
                name: "max".to_string(),
                params: vec![],
            }),
            operator: Operator::Plus,
        };

        let result = validate_env(&StaticEnvironment::default(), &ast);

        assert_eq!(Err(Error::MissingFunction("max".to_string())), result);
    }

    fn dummy_function(_params: &[Value]) -> Result<Value, String> {
        Ok(Value::Boolean(true))
    }

    #[test]
    fn err_function_params_mismatch() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Call {
                name: "max".to_string(),
                params: vec![],
            }),
            operator: Operator::Plus,
        };

        let mut env = StaticEnvironment::default();
        env.add_native_func("max", Some(2), dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(
            Err(Error::ParamCountMismatch("max".to_string(), 0, 2)),
            result
        );
    }

    #[test]
    fn err_function_nested_params() {
        let ast = Expression::Call {
            name: "func".to_string(),
            params: vec![Expression::Variable {
                name: "not_found".to_string(),
            }],
        };

        let mut env = StaticEnvironment::default();
        env.add_native_func("func", None, dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(Err(Error::MissingVariable("not_found".to_string())), result);
    }
}
