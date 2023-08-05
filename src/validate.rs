use crate::{
    ast::Expression,
    environment::{Environment, FunctionResult},
    operator::Operator,
    value::Value,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    MissingVariable(String),
    MissingFunction(String),
    ParamCountMismatch(usize),
    InvalidOperator(String),
    LiteralNotBoolean,
}

/// Validates `Variable` and `Call` [`Expressions`](crate::ast::Expression) by walking
/// the tree and returning a [`ValidationResult`] on the first error.
pub fn validate_env(env: &dyn Environment, expression: &Expression) -> ValidationResult {
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
        Expression::Array {
            expressions: values,
        } => result = validate_expr_vec(env, values),
        Expression::Variable { name } => {
            if env.variable(name).is_none() {
                result = ValidationResult::MissingVariable(name.clone());
            }
        }
        Expression::Call { name, params } => {
            result = match env.function(name, params.len()) {
                FunctionResult::Exists => validate_expr_vec(env, params),
                FunctionResult::NotFound => ValidationResult::MissingFunction(name.clone()),
                FunctionResult::WrongArity => ValidationResult::ParamCountMismatch(params.len()),
            }
        }
        Expression::Literal { value: _ } => (),
    };

    result
}

fn validate_expr_vec(env: &dyn Environment, expressions: &Vec<Expression>) -> ValidationResult {
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

/// Checks if the top level [`Expression`] produces a [`Value::Boolean`] result.
///
/// # Examples
///
/// ```
/// use slac::validate::{validate_boolean_result, ValidationResult};
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
/// assert_eq!(validate_boolean_result(&ast), ValidationResult::Valid);
/// ```
pub fn validate_boolean_result(ast: &Expression) -> ValidationResult {
    match ast {
        Expression::Unary { right: _, operator } => match operator {
            Operator::Not => ValidationResult::Valid,
            _ => ValidationResult::InvalidOperator(operator.to_string()),
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
            | Operator::Or => ValidationResult::Valid,
            _ => ValidationResult::InvalidOperator(operator.to_string()),
        },
        Expression::Array { expressions: _ } => ValidationResult::LiteralNotBoolean,
        Expression::Literal { value } => match value {
            Value::Boolean(_) => ValidationResult::Valid,
            _ => ValidationResult::LiteralNotBoolean,
        },
        Expression::Variable { name: _ } => ValidationResult::Valid, // Type not known
        Expression::Call { name: _, params: _ } => ValidationResult::Valid, // Type not known
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression, environment::StaticEnvironment, operator::Operator,
        validate::ValidationResult, value::Value,
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

        assert_eq!(ValidationResult::Valid, result);
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

        assert_eq!(
            ValidationResult::MissingVariable("VAR_NAME".to_string()),
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
                name: "max".to_string(),
                params: vec![],
            }),
            operator: Operator::Plus,
        };

        let result = validate_env(&StaticEnvironment::default(), &ast);

        assert_eq!(ValidationResult::MissingFunction("max".to_string()), result);
    }

    fn dummy_function(_params: Vec<Value>) -> Result<Value, String> {
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
        env.add_native_func("max", 2, dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(ValidationResult::ParamCountMismatch(0), result);
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
        env.add_native_func("func", 1, dummy_function);

        let result = validate_env(&env, &ast);

        assert_eq!(
            ValidationResult::MissingVariable("not_found".to_string()),
            result
        );
    }
}
