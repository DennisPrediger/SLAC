//! Transformation routines to optimize an [`Expression`] AST.

use std::{cell::RefCell, rc::Rc};

use crate::{execute, Expression, Operator, Result, StaticEnvironment};

/// Recursivly transforms ternary function calls into [`Expression::Ternary`].
/// Three parameter [`crate::stdlib::common::if_then`] calls are transformed
/// into a [`Operator::TernaryCondition`];
///
/// # Remarks
///
/// While the [`crate::stdlib::common::if_then`] is eagerly evaluated, the
/// [`Expression::Ternary`] supports short-circuit evaluation in the `TreeWalkingInterpreter`.
pub fn transform_ternary(expr: Expression) -> Expression {
    match expr {
        Expression::Unary { right, operator } => Expression::Unary {
            right: Box::new(transform_ternary(*right)),
            operator,
        },
        Expression::Binary {
            left,
            right,
            operator,
        } => Expression::Binary {
            left: Box::new(transform_ternary(*left)),
            right: Box::new(transform_ternary(*right)),
            operator,
        },
        Expression::Ternary {
            left,
            middle,
            right,
            operator,
        } => Expression::Ternary {
            left: Box::new(transform_ternary(*left)),
            middle: Box::new(transform_ternary(*middle)),
            right: Box::new(transform_ternary(*right)),
            operator,
        },
        Expression::Array { expressions } => Expression::Array {
            expressions: expressions
                .iter()
                .map(|e| transform_ternary(e.clone()))
                .collect(),
        },
        Expression::Call {
            ref name,
            ref params,
        } if name == "if_then" => match params.as_slice() {
            // only match on three parameter "if_then"
            [left, middle, right] => Expression::Ternary {
                left: Box::new(transform_ternary(left.clone())),
                middle: Box::new(transform_ternary(middle.clone())),
                right: Box::new(transform_ternary(right.clone())),
                operator: Operator::TernaryCondition,
            },
            _ => Expression::Call {
                name: name.clone(),
                params: params
                    .iter()
                    .map(|e| transform_ternary(e.clone()))
                    .collect(),
            },
        },
        _ => expr,
    }
}

/// Evaluates [`Expression::Unary`] and [`Expression::Binary`] into a single
/// [`Expression::Literal`] if all arguments are also an [`Expression::Literal`].
///
/// Evaluates [`Operator::TernaryCondition`] [`Expression::Ternary`] into either
/// the second or third argument, if the first argument is a [`Expression::Literal`].
///
/// # Remarks
///
/// This function is repeatedly applied to the expression until no further folding is possible.
///
/// # Errors
///
/// Will return [`crate::Error`] if the Evaluation is not possible.
pub fn fold_constants(mut expression: Expression) -> Result<Expression> {
    fn try_fold(expr: Expression, found_const: &Rc<RefCell<bool>>) -> Result<Expression> {
        match expr {
            Expression::Unary {
                ref right,
                ref operator,
            } => match right.as_ref() {
                Expression::Literal { value: _ } => {
                    found_const.replace(true);
                    Ok(Expression::Literal {
                        value: execute(&StaticEnvironment::default(), &expr)?,
                    })
                }
                _ => Ok(Expression::Unary {
                    right: Box::new(try_fold(*right.clone(), found_const)?),
                    operator: operator.clone(),
                }),
            },
            Expression::Binary {
                ref left,
                ref right,
                ref operator,
            } => match (left.as_ref(), right.as_ref()) {
                (Expression::Literal { value: _ }, Expression::Literal { value: _ }) => {
                    found_const.replace(true);
                    Ok(Expression::Literal {
                        value: execute(&StaticEnvironment::default(), &expr)?,
                    })
                }
                _ => Ok(Expression::Binary {
                    left: Box::new(try_fold(*left.clone(), found_const)?),
                    right: Box::new(try_fold(*right.clone(), found_const)?),
                    operator: operator.clone(),
                }),
            },
            Expression::Ternary {
                left,
                middle,
                right,
                operator,
            } => match (left.as_ref(), operator) {
                (Expression::Literal { value }, Operator::TernaryCondition) => {
                    found_const.replace(true);
                    if value.as_bool() {
                        Ok(*middle.clone())
                    } else {
                        Ok(*right.clone())
                    }
                }
                _ => Ok(Expression::Ternary {
                    left: Box::new(try_fold(*left, found_const)?),
                    middle: Box::new(try_fold(*middle, found_const)?),
                    right: Box::new(try_fold(*right, found_const)?),
                    operator,
                }),
            },
            Expression::Array { expressions } => Ok(Expression::Array {
                expressions: expressions
                    .iter()
                    .map(|e| try_fold(e.clone(), found_const))
                    .collect::<Result<Vec<_>>>()?,
            }),
            Expression::Call { name, params } => Ok(Expression::Call {
                name,
                params: params
                    .iter()
                    .map(|e| try_fold(e.clone(), found_const))
                    .collect::<Result<Vec<_>>>()?,
            }),
            _ => Ok(expr),
        }
    }

    let found_const = Rc::new(RefCell::new(false));

    loop {
        // repeat until no further folding is possible
        expression = try_fold(expression, &found_const)?;

        if found_const.replace(false) == false {
            return Ok(expression);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{fold_constants, transform_ternary};
    use crate::{Expression, Operator, Value};

    #[test]
    fn ternary_flat() {
        let expr = Expression::Call {
            name: String::from("if_then"),
            params: vec![
                Expression::Literal {
                    value: Value::Boolean(true),
                },
                Expression::Literal {
                    value: Value::Number(1.0),
                },
                Expression::Literal {
                    value: Value::Number(2.0),
                },
            ],
        };

        let ternary = Expression::Ternary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            middle: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::TernaryCondition,
        };

        assert_eq!(ternary, transform_ternary(expr));
    }

    #[test]
    fn ternary_nested() {
        let expr = Expression::Unary {
            right: Box::new(Expression::Call {
                name: String::from("if_then"),
                params: vec![
                    Expression::Literal {
                        value: Value::Boolean(true),
                    },
                    Expression::Literal {
                        value: Value::Number(1.0),
                    },
                    Expression::Literal {
                        value: Value::Number(2.0),
                    },
                ],
            }),
            operator: Operator::Minus,
        };

        let ternary = Expression::Unary {
            right: Box::new(Expression::Ternary {
                left: Box::new(Expression::Literal {
                    value: Value::Boolean(true),
                }),
                middle: Box::new(Expression::Literal {
                    value: Value::Number(1.0),
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Number(2.0),
                }),
                operator: Operator::TernaryCondition,
            }),
            operator: Operator::Minus,
        };

        assert_eq!(ternary, transform_ternary(expr));
    }

    #[test]
    fn fold_const_flat_binary() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(5.0),
            }),
            operator: Operator::Plus,
        };

        let value = Expression::Literal {
            value: Value::Number(15.0),
        };

        assert_eq!(Ok(value), fold_constants(expr));
    }

    #[test]
    fn fold_const_flat_unary() {
        let expr = Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Number(5.0),
            }),
            operator: Operator::Minus,
        };

        let value = Expression::Literal {
            value: Value::Number(-5.0),
        };

        assert_eq!(Ok(value), fold_constants(expr));

        let expr = Expression::Unary {
            right: Box::new(Expression::Unary {
                right: Box::new(Expression::Literal {
                    value: Value::Number(5.0),
                }),
                operator: Operator::Minus,
            }),
            operator: Operator::Minus,
        };

        let value = Expression::Literal {
            value: Value::Number(5.0),
        };

        assert_eq!(Ok(value), fold_constants(expr));
    }

    #[test]
    fn fold_const_ternary() {
        let expr = Expression::Ternary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            middle: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::TernaryCondition,
        };

        let value = Expression::Literal {
            value: Value::Number(1.0),
        };

        assert_eq!(Ok(value), fold_constants(expr));
    }
}
