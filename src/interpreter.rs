use crate::{ast::Expression, environment::Environment, token::Token, value::Value};

pub struct TreeWalkingInterpreter<'a> {
    environment: &'a Environment,
}

impl<'a> TreeWalkingInterpreter<'a> {
    pub fn new(environment: &'a Environment) -> Self {
        Self { environment }
    }

    pub fn interprete(env: &Environment, expression: &Expression) -> Value {
        TreeWalkingInterpreter::new(env).expression(expression)
    }

    fn expression(&self, expression: &Expression) -> Value {
        match expression {
            Expression::Unary { right, operator } => self.unary(right, operator),
            Expression::Binary {
                left,
                right,
                operator,
            } => self.binary(left, right, operator),
            Expression::Literal(value) => value.clone(),
            Expression::Variable(name) => self.variable(name),
            Expression::Call(name, params) => self.call(name, params),
        }
    }

    fn unary(&self, right: &Expression, operator: &Token) -> Value {
        let right = self.expression(right);

        match operator {
            Token::Minus => -right,
            Token::Not => !right,
            _ => Value::Boolean(false),
        }
    }

    fn binary(&self, left: &Expression, right: &Expression, operator: &Token) -> Value {
        let left = self.expression(left);
        let right = self.expression(right);

        match operator {
            Token::Plus => left + right,
            Token::Minus => left - right,
            Token::Star => left * right,
            Token::Slash => left / right,
            Token::Div => left.div_int(right),
            Token::Mod => left % right,
            Token::Greater => Value::Boolean(left > right),
            Token::GreaterEqual => Value::Boolean(left >= right),
            Token::Less => Value::Boolean(left < right),
            Token::LessEqual => Value::Boolean(left <= right),
            Token::Equal => Value::Boolean(left == right),
            Token::NotEqual => Value::Boolean(left != right),
            Token::And => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs && rhs),
                _ => Value::Boolean(false),
            },
            Token::Or => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs || rhs),
                _ => Value::Boolean(false),
            },
            _ => Value::Boolean(false),
        }
    }

    fn variable(&self, name: &str) -> Value {
        self.environment
            .get_var(name)
            .unwrap_or(&Value::Boolean(false))
            .clone()
    }

    fn call(&self, name: &str, params: &Vec<Expression>) -> Value {
        match self.environment.get_func(name, params.len()) {
            Some(func) => {
                let params = params
                    .iter()
                    .map(|expression| self.expression(expression))
                    .collect();

                func(params).unwrap_or(Value::Boolean(false))
            }
            None => Value::Boolean(false),
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::{ast::Expression, interpreter::TreeWalkingInterpreter, token::Token, value::Value};

    use super::Environment;

    #[test]
    fn bool_not() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Boolean(false))),
            operator: Token::Not,
        };
        let env = Environment::new();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn number_minus() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Number(42.0))),
            operator: Token::Minus,
        };
        let env = Environment::new();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Number(-42.0), value);
    }

    #[test]
    fn bool_and_true() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal(Value::Boolean(true))),
            right: Box::from(Expression::Literal(Value::Boolean(true))),
            operator: Token::And,
        };
        let env = Environment::new();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn bool_and_false() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal(Value::Boolean(true))),
            right: Box::from(Expression::Literal(Value::Boolean(false))),
            operator: Token::And,
        };
        let env = Environment::new();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(false), value);
    }

    #[test]
    fn variable_access() {
        let ast = Expression::Variable("test".to_string());
        let mut env = Environment::new();

        env.add_var("test".to_string(), Value::Number(42.0));
        let result = TreeWalkingInterpreter::interprete(&env, &ast);
        let expected = Value::Number(42.0);

        assert_eq!(expected, result);
    }

    fn max(params: Vec<Value>) -> Result<Value, String> {
        let result = params
            .iter()
            .max_by(|a, b| {
                if a > b {
                    Ordering::Greater
                } else if a < b {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .unwrap()
            .clone();

        Ok(result)
    }

    #[test]
    fn func_access() {
        let ast = Expression::Call(
            String::from("max"),
            vec![
                Expression::Literal(Value::Number(10.0)),
                Expression::Literal(Value::Number(20.0)),
            ],
        );

        let mut env = Environment::new();
        env.add_native_func(String::from("max"), 2, max);

        let result = TreeWalkingInterpreter::interprete(&env, &ast);
        let expected = Value::Number(20.0);
        assert_eq!(expected, result);
    }
}
