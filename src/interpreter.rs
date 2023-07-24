use std::collections::HashMap;

use crate::{ast::Expression, token::Token, value::Value};

type NativeFunction = fn(Vec<Value>) -> Result<Value, String>;

pub struct Environment {
    constants: HashMap<String, Value>,
    functions: HashMap<String, NativeFunction>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn add_const(&mut self, name: String, value: Value) {
        self.constants.insert(name, value);
    }

    pub fn add_native_func(&mut self, name: String, func: NativeFunction) {
        self.functions.insert(name, func);
    }

    pub fn interprete(&self, expression: &Expression) -> Value {
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
        let right = self.interprete(right);

        match operator {
            Token::Minus => -right,
            Token::Not => !right,
            _ => Value::Boolean(false),
        }
    }

    fn binary(&self, left: &Expression, right: &Expression, operator: &Token) -> Value {
        let left = self.interprete(left);
        let right = self.interprete(right);

        match operator {
            Token::Plus => left + right,
            Token::Minus => left - right,
            Token::Star => left * right,
            Token::Slash => left / right,
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
        self.constants
            .get(name)
            .unwrap_or(&Value::Boolean(false))
            .clone()
    }

    fn call(&self, name: &str, params: &Vec<Expression>) -> Value {
        match self.functions.get(name) {
            Some(func) => {
                let params = params
                    .iter()
                    .map(|expression| self.interprete(expression))
                    .collect();

                func(params).unwrap_or(Value::Boolean(false))
            }
            None => Value::Boolean(false),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Expression, token::Token, value::Value};

    use super::Environment;

    #[test]
    fn bool_not() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Boolean(false))),
            operator: Token::Not,
        };
        let env = Environment::new();
        let value = env.interprete(&ast);

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn number_minus() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Number(42.0))),
            operator: Token::Minus,
        };
        let env = Environment::new();
        let value = env.interprete(&ast);

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
        let value = env.interprete(&ast);

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
        let value = env.interprete(&ast);

        assert_eq!(Value::Boolean(false), value);
    }
}
