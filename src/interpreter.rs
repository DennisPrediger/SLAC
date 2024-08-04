use crate::{
    ast::Expression, environment::Environment, operator::Operator, value::Value, Error, Result,
};

/// A simple recursive tree walking interpreter.
/// Given an [`Environment`] and an [`AST`](Expression) recursivly walks the tree
/// and computes a single output [`Value`].
///
/// # Errors
///
/// Returns an [`Error`] if the execution fails.
#[allow(clippy::module_name_repetitions)]
pub struct TreeWalkingInterpreter<'a> {
    environment: &'a dyn Environment,
}

impl<'a> TreeWalkingInterpreter<'a> {
    pub fn new(environment: &'a dyn Environment) -> Self {
        Self { environment }
    }

    pub fn interprete(env: &impl Environment, expression: &Expression) -> Result<Value> {
        TreeWalkingInterpreter::new(env).expression(expression)
    }

    fn expression(&self, expression: &Expression) -> Result<Value> {
        match expression {
            Expression::Unary { right, operator } => self.unary(right, *operator),
            Expression::Binary {
                left,
                right,
                operator,
            } => self.binary(left, right, *operator),
            Expression::Ternary {
                left,
                middle,
                right,
                operator,
            } => self.ternary(left, middle, right, *operator),
            Expression::Array { expressions } => self.array(expressions),
            Expression::Literal { value } => Ok(value.clone()),
            Expression::Variable { name } => self.variable(name),
            Expression::Call { name, params } => self.call(name, params),
        }
    }

    fn unary(&self, right: &Expression, operator: Operator) -> Result<Value> {
        let right = self.expression(right);

        match (operator, right) {
            (Operator::Minus, Ok(rhs)) => -rhs,
            (Operator::Not, Ok(rhs)) => !rhs,
            _ => Err(Error::InvalidUnaryOperator(operator)),
        }
    }

    fn binary(&self, left: &Expression, right: &Expression, operator: Operator) -> Result<Value> {
        let left = self.expression(left);

        match (operator, left) {
            (Operator::And, Ok(left)) => self.boolean::<true>(&left, right),
            (Operator::Or, Ok(left)) => self.boolean::<false>(&left, right),
            (_, Ok(left)) => {
                let right = self.expression(right);

                match (operator, right) {
                    (Operator::Plus, Ok(right)) => left + right,
                    (Operator::Minus, Ok(right)) => left - right,
                    (Operator::Multiply, Ok(right)) => left * right,
                    (Operator::Divide, Ok(right)) => left / right,
                    (Operator::Div, Ok(right)) => left.div_int(right),
                    (Operator::Mod, Ok(right)) => left % right,
                    (Operator::Xor, Ok(right)) => left ^ right,
                    (Operator::Greater, Ok(right)) => Ok(Value::Boolean(left > right)),
                    (Operator::GreaterEqual, Ok(right)) => Ok(Value::Boolean(left >= right)),
                    (Operator::Less, Ok(right)) => Ok(Value::Boolean(left < right)),
                    (Operator::LessEqual, Ok(right)) => Ok(Value::Boolean(left <= right)),
                    (Operator::Equal, Ok(right)) => Ok(Value::Boolean(left == right)),
                    (Operator::NotEqual, Ok(right)) => Ok(Value::Boolean(left != right)),
                    (Operator::Equal, Err(Error::UndefinedVariable(_))) => {
                        // Check if the left expression is equal to empty
                        Ok(Value::Boolean(left.is_empty()))
                    }
                    (Operator::NotEqual, Err(Error::UndefinedVariable(_))) => {
                        // Check if the left expression is not equal to empty
                        Ok(Value::Boolean(!left.is_empty()))
                    }
                    (_, Err(right)) => Err(right),
                    (operator, _) => Err(Error::InvalidBinaryOperator(operator)),
                }
            }
            (Operator::Equal, Err(Error::UndefinedVariable(_))) => {
                // Check if the right expression is equal to empty
                match self.expression(right) {
                    Ok(right) => Ok(Value::Boolean(right.is_empty())),
                    // check `empty = empty -> true`
                    Err(Error::UndefinedVariable(_)) => Ok(Value::Boolean(true)),
                    Err(right) => Err(right),
                }
            }
            (Operator::NotEqual, Err(Error::UndefinedVariable(_))) => {
                // Check if the right expression is not equal to empty
                match self.expression(right) {
                    Ok(right) => Ok(Value::Boolean(!right.is_empty())),
                    // check `empty <> empty -> true`
                    Err(Error::UndefinedVariable(_)) => Ok(Value::Boolean(false)),
                    Err(right) => Err(right),
                }
            }
            (_, Err(left)) => Err(left),
        }
    }

    fn boolean<const FULL_EVAL: bool>(&self, left: &Value, right: &Expression) -> Result<Value> {
        let left = left.as_bool();

        if left == FULL_EVAL {
            let right = self.expression(right)?;
            Ok(Value::Boolean(right.as_bool()))
        } else {
            Ok(Value::Boolean(left)) // short circuit
        }
    }

    fn ternary(
        &self,
        left: &Expression,
        middle: &Expression,
        right: &Expression,
        operator: Operator,
    ) -> Result<Value> {
        match operator {
            Operator::TernaryCondition => {
                let left = self.expression(left)?;

                // short circuit evaluation
                if left.as_bool() {
                    self.expression(middle)
                } else {
                    self.expression(right)
                }
            }
            _ => Err(Error::InvalidTernaryOperator(operator)),
        }
    }

    fn get_values(&self, expressions: &[Expression]) -> Result<Vec<Value>> {
        expressions
            .iter()
            .map(|expression| self.expression(expression))
            .collect::<Result<_>>()
    }

    fn array(&self, expressions: &[Expression]) -> Result<Value> {
        Ok(Value::Array(self.get_values(expressions)?))
    }

    fn variable(&self, name: &str) -> Result<Value> {
        self.environment
            .variable(name)
            .map(|v| (*v).clone())
            .ok_or(Error::UndefinedVariable(name.to_string()))
    }

    fn call(&self, name: &str, expressions: &[Expression]) -> Result<Value> {
        self.environment
            .call(name, &self.get_values(expressions)?)
            .map_err(|e| Error::NativeFunctionError(name.to_string(), e))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression,
        function::{Arity, Function},
        interpreter::TreeWalkingInterpreter,
        operator::Operator,
        stdlib::common::max,
        value::Value,
        StaticEnvironment,
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

        env.add_variable("test", Value::Number(42.0));
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
        env.add_function(Function::new(
            max,
            Arity::Polyadic {
                required: 2,
                optional: 0,
            },
            "max(left: Number, right: Number): Number",
        ));

        let result = TreeWalkingInterpreter::interprete(&env, &ast).unwrap();
        let expected = Value::Number(20.0);
        assert_eq!(expected, result);
    }
}
