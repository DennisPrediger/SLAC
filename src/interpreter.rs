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

    pub fn interprete(env: &dyn Environment, expression: &Expression) -> Option<Value> {
        TreeWalkingInterpreter::new(env).expression(expression)
    }

    fn expression(&self, expression: &Expression) -> Option<Value> {
        match expression {
            Expression::Unary { right, operator } => self.unary(right, operator),
            Expression::Binary {
                left,
                right,
                operator,
            } => self.binary(left, right, operator),
            Expression::Array { expressions } => self.array(expressions),
            Expression::Literal { value } => Some(value.clone()),
            Expression::Variable { name } => self.variable(name),
            Expression::Call { name, params } => self.call(name, params),
        }
    }

    fn unary(&self, right: &Expression, operator: &Operator) -> Option<Value> {
        let right = self.expression(right);

        match (operator, right) {
            (Operator::Minus, Some(rhs)) => -rhs,
            (Operator::Not, Some(rhs)) => !rhs,
            _ => None,
        }
    }

    fn binary(&self, left: &Expression, right: &Expression, operator: &Operator) -> Option<Value> {
        let left = self.expression(left);

        match (operator, left) {
            (Operator::And, Some(left)) => self.boolean(left, right, true),
            (Operator::Or, Some(left)) => self.boolean(left, right, false),
            (_, Some(left)) => {
                let right = self.expression(right);

                match (operator, right) {
                    (Operator::Plus, Some(right)) => left + right,
                    (Operator::Minus, Some(right)) => left - right,
                    (Operator::Multiply, Some(right)) => left * right,
                    (Operator::Divide, Some(right)) => left / right,
                    (Operator::Div, Some(right)) => left.div_int(right),
                    (Operator::Mod, Some(right)) => left % right,
                    (Operator::Xor, Some(right)) => left ^ right,
                    (Operator::Greater, Some(right)) => Some(Value::Boolean(left > right)),
                    (Operator::GreaterEqual, Some(right)) => Some(Value::Boolean(left >= right)),
                    (Operator::Less, Some(right)) => Some(Value::Boolean(left < right)),
                    (Operator::LessEqual, Some(right)) => Some(Value::Boolean(left <= right)),
                    (Operator::Equal, Some(right)) => Some(Value::Boolean(left == right)),
                    (Operator::NotEqual, Some(right)) => Some(Value::Boolean(left != right)),
                    (Operator::Equal, None) => Some(Value::Boolean(left.is_empty())),
                    (Operator::NotEqual, None) => Some(Value::Boolean(!left.is_empty())),
                    _ => None,
                }
            }
            (Operator::Equal, None) => match self.expression(right) {
                Some(right) => Some(Value::Boolean(right.is_empty())),
                None => Some(Value::Boolean(true)),
            },
            (Operator::NotEqual, None) => match self.expression(right) {
                Some(right) => Some(Value::Boolean(!right.is_empty())),
                None => Some(Value::Boolean(true)),
            },
            _ => None,
        }
    }

    fn boolean(&self, left: Value, right: &Expression, full_evaluate_on: bool) -> Option<Value> {
        match left {
            Value::Boolean(left) => {
                if left == full_evaluate_on {
                    // if `left` is not the result we need, evaluate `right`
                    match self.expression(right) {
                        Some(Value::Boolean(right)) => Some(Value::Boolean(right)),
                        _ => None,
                    }
                } else {
                    Some(Value::Boolean(left)) // short circuit
                }
            }
            _ => None,
        }
    }

    fn array(&self, expressions: &Vec<Expression>) -> Option<Value> {
        let mut values: Vec<Value> = vec![];

        for expression in expressions {
            values.push(self.expression(expression)?);
        }

        Some(Value::Array(values))
    }

    fn variable(&self, name: &str) -> Option<Value> {
        self.environment.variable(name).map(|v| (*v).clone())
    }

    fn call(&self, name: &str, params: &[Expression]) -> Option<Value> {
        let params: Option<Vec<Value>> = params
            .iter()
            .map(|expression| self.expression(expression))
            .collect();

        self.environment.call(name, &params?)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression, interpreter::TreeWalkingInterpreter, operator::Operator, std::common::max,
        value::Value, StaticEnvironment,
    };

    #[test]
    fn bool_not() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::Not,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn number_minus() {
        let ast = Expression::Unary {
            right: Box::from(Expression::Literal {
                value: Value::Number(42.0),
            }),
            operator: Operator::Minus,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();

        assert_eq!(Value::Number(-42.0), value);
    }

    #[test]
    fn bool_and_true() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal {
                value: Value::Boolean(true),
            }),
            right: Box::from(Expression::Literal {
                value: Value::Boolean(true),
            }),
            operator: Operator::And,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();

        assert_eq!(Value::Boolean(true), value);
    }

    #[test]
    fn bool_and_false() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Literal {
                value: Value::Boolean(true),
            }),
            right: Box::from(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::And,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();

        assert_eq!(Value::Boolean(false), value);
    }

    #[test]
    fn array_plus_array() {
        let ast = Expression::Binary {
            left: Box::from(Expression::Array {
                expressions: vec![
                    Expression::Literal {
                        value: Value::Number(10.0),
                    },
                    Expression::Literal {
                        value: Value::Number(20.0),
                    },
                ],
            }),
            right: Box::from(Expression::Array {
                expressions: vec![
                    Expression::Literal {
                        value: Value::Number(30.0),
                    },
                    Expression::Literal {
                        value: Value::Number(40.0),
                    },
                ],
            }),
            operator: Operator::Plus,
        };
        let env = StaticEnvironment::default();
        let value = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();

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
        let ast = Expression::Variable {
            name: String::from("test"),
        };
        let mut env = StaticEnvironment::default();

        env.add_var("test", Value::Number(42.0));
        let result = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();
        let expected = Value::Number(42.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn func_access() {
        let ast = Expression::Call {
            name: String::from("max"),
            params: vec![
                Expression::Literal {
                    value: Value::Number(10.0),
                },
                Expression::Literal {
                    value: Value::Number(20.0),
                },
            ],
        };

        let mut env = StaticEnvironment::default();
        env.add_native_func("max", Some(2), 0, max);

        let result = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();
        let expected = Value::Number(20.0);
        assert_eq!(expected, result);
    }
}
