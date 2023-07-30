use crate::{ast::Expression, environment::Environment};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    MissingVariable(String),
    MissingFunction(String),
    ParamCountMismatch(usize, usize),
}

/// Validates `Variable` and `Call` [`Expressions`](crate::ast::Expression) by walking
/// the tree and returning a [`ValidationResult`] on the first error.
pub fn validate_env(env: &Environment, expression: &Expression) -> ValidationResult {
    let mut result = ValidationResult::Valid;

    match expression {
        Expression::Unary { right, operator: _ } => result = validate_env(env, right),
        Expression::Binary {
            left,
            right,
            operator: _,
        } => {
            result = validate_env(env, left);
            if let ValidationResult::Valid = result {
                result = validate_env(env, right);
            }
        }
        Expression::Array(expressions) => result = validate_expr_vec(env, expressions),
        Expression::Variable(name) => {
            if env.get_var(name).is_none() {
                result = ValidationResult::MissingVariable(name.clone());
            }
        }
        Expression::Call(name, params) => match env.get_function(name) {
            Some(function) => {
                if function.arity != params.len() {
                    result = ValidationResult::ParamCountMismatch(function.arity, params.len());
                } else {
                    result = validate_expr_vec(env, params);
                }
            }
            None => result = ValidationResult::MissingFunction(name.clone()),
        },
        Expression::Literal(_) => (),
    };

    result
}

fn validate_expr_vec(env: &Environment, expressions: &Vec<Expression>) -> ValidationResult {
    let mut result = ValidationResult::Valid;

    for expr in expressions {
        result = validate_env(env, expr);
        if let ValidationResult::Valid = result {
            // do nothing
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression, environment::Environment, token::Token, validate::ValidationResult,
        value::Value,
    };

    use super::validate_env;

    #[test]
    fn valid() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Number(10.0))),
            right: Box::new(Expression::Literal(Value::Number(10.0))),
            operator: Token::Plus,
        };

        let result = validate_env(&Environment::default(), &ast);

        assert_eq!(ValidationResult::Valid, result);
    }

    #[test]
    fn err_missing_variable() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Number(10.0))),
            right: Box::new(Expression::Variable("VAR_NAME".to_string())),
            operator: Token::Plus,
        };

        let result = validate_env(&Environment::default(), &ast);

        assert_eq!(
            ValidationResult::MissingVariable("VAR_NAME".to_string()),
            result
        );
    }

    #[test]
    fn err_function_missing() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Number(10.0))),
            right: Box::new(Expression::Call("max".to_string(), vec![])),
            operator: Token::Plus,
        };

        let result = validate_env(&Environment::default(), &ast);

        assert_eq!(ValidationResult::MissingFunction("max".to_string()), result);
    }

    fn dummy_function(_params: Vec<Value>) -> Result<Value, String> {
        Ok(Value::Boolean(true))
    }

    #[test]
    fn err_function_params_mismatch() {
        let ast = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Number(10.0))),
            right: Box::new(Expression::Call("max".to_string(), vec![])),
            operator: Token::Plus,
        };

        let mut env = Environment::default();
        env.add_native_func("max", 2, dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(ValidationResult::ParamCountMismatch(2, 0), result);
    }

    #[test]
    fn err_function_nested_params() {
        let ast = Expression::Call(
            "func".to_string(),
            vec![Expression::Variable("not_found".to_string())],
        );

        let mut env = Environment::default();
        env.add_native_func("func", 1, dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(
            ValidationResult::MissingVariable("not_found".to_string()),
            result
        );
    }
}
