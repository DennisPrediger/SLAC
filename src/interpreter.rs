use std::vec;

use crate::{ast::Expression, environment::Environment, operator::Operator, value::Value};

/// A tree walking interpreter which given an [`Environment`] and an [`AST`](Expression)
/// recursivly walks the tree and computes a single [`Value`].
pub struct TreeWalkingInterpreter<'a> {
    environment: &'a dyn Environment,
}

impl<'a> TreeWalkingInterpreter<'a> {
    pub fn new(environment: &'a dyn Environment) -> Self {
        Self { environment }
    }

    pub fn interprete(env: &dyn Environment, expression: &Expression) -> Value {
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
            Expression::Array(expressions) => self.array(expressions),
            Expression::Literal(value) => value.clone(),
            Expression::Variable(name) => self.variable(name),
            Expression::Call(name, params) => self.call(name, params),
        }
    }

    fn unary(&self, right: &Expression, operator: &Operator) -> Value {
        let right = self.expression(right);

        match operator {
            Operator::Minus => -right,
            Operator::Not => !right,
            _ => Value::Boolean(false),
        }
    }

    fn binary(&self, left: &Expression, right: &Expression, operator: &Operator) -> Value {
        let left = self.expression(left);
        let right = self.expression(right);

        match operator {
            Operator::Plus => left + right,
            Operator::Minus => left - right,
            Operator::Star => left * right,
            Operator::Slash => left / right,
            Operator::Div => left.div_int(right),
            Operator::Mod => left % right,
            Operator::Greater => Value::Boolean(left > right),
            Operator::GreaterEqual => Value::Boolean(left >= right),
            Operator::Less => Value::Boolean(left < right),
            Operator::LessEqual => Value::Boolean(left <= right),
            Operator::Equal => Value::Boolean(left == right),
            Operator::NotEqual => Value::Boolean(left != right),
            Operator::And => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs && rhs),
                _ => Value::Boolean(false),
            },
            Operator::Or => match (left, right) {
                (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs || rhs),
                _ => Value::Boolean(false),
            },
            _ => Value::Boolean(false),
        }
    }

    fn array(&self, expressions: &Vec<Expression>) -> Value {
        let mut values: Vec<Value> = vec![];

        for expression in expressions {
            values.push(self.expression(expression));
        }

        Value::Array(values)
    }

    fn variable(&self, name: &str) -> Value {
        self.environment
            .variable(name)
            .map(|v| (*v).clone())
            .unwrap_or(Value::Nil)
    }

    fn call(&self, name: &str, params: &Vec<Expression>) -> Value {
        match self.environment.function(name) {
            Some(function) if function.arity == params.len() => {
                let func = function.func;
                let params = params
                    .iter()
                    .map(|expression| self.expression(expression))
                    .collect();

                func(params).unwrap_or(Value::Nil)
            }
            _ => Value::Nil,
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use crate::{
        ast::Expression, environment::StaticEnvironment, interpreter::TreeWalkingInterpreter,
        operator::Operator, value::Value,
    };

    #[test]
    fn bool_not() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Boolean(false))),
            operator: Operator::Not,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn number_minus() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal(Value::Number(42.0))),
            operator: Operator::Minus,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Number(-42.0), value);
    }

    #[test]
    fn bool_and_true() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal(Value::Boolean(true))),
            right: Box::from(Expression::Literal(Value::Boolean(true))),
            operator: Operator::And,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn bool_and_false() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal(Value::Boolean(true))),
            right: Box::from(Expression::Literal(Value::Boolean(false))),
            operator: Operator::And,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(Value::Boolean(false), value);
    }

    #[test]
    fn array_plus_array() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Array(vec![
                Expression::Literal(Value::Number(10.0)),
                Expression::Literal(Value::Number(20.0)),
            ])),
            right: Box::from(Expression::Array(vec![
                Expression::Literal(Value::Number(30.0)),
                Expression::Literal(Value::Number(40.0)),
            ])),
            operator: Operator::Plus,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast);

        assert_eq!(
            Value::Array(vec![
                Value::Number(10.0),
                Value::Number(20.0),
                Value::Number(30.0),
                Value::Number(40.0)
            ]),
            value
        );
    }

    #[test]
    fn variable_access() {
        let ast = Expression::Variable("test".to_string());
        let mut env = StaticEnvironment::default();

        env.add_var("test", Value::Number(42.0));
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

        let mut env = StaticEnvironment::default();
        env.add_native_func("max", 2, max);

        let result = TreeWalkingInterpreter::interprete(&env, &ast);
        let expected = Value::Number(20.0);
        assert_eq!(expected, result);
    }
}
