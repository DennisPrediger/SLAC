//! Transformation routines to optimize an [`Expression`] AST.

use crate::environment::{Environment, FunctionResult};
use crate::{execute, Expression, Operator, Result};

use crate::stdlib::common::TERNARY_IF_THEN;

/// Recursivly transforms ternary function calls into [`Expression::Ternary`].
/// Three parameter [`crate::stdlib::common::if_then`] calls are transformed
/// into a [`Operator::TernaryCondition`];
///
/// # Remarks
///
/// While the [`crate::stdlib::common::if_then`] is eagerly evaluated, the
/// [`Expression::Ternary`] supports short-circuit evaluation in the `TreeWalkingInterpreter`.
pub fn transform_ternary(expression: &mut Expression, found_const: &mut bool) {
    match expression {
        Expression::Unary { right, operator: _ } => {
            transform_ternary(right, found_const);
        }
        Expression::Binary {
            left,
            right,
            operator: _,
        } => {
            transform_ternary(left, found_const);
            transform_ternary(right, found_const);
        }
        Expression::Ternary {
            left,
            middle,
            right,
            operator: _,
        } => {
            transform_ternary(left, found_const);
            transform_ternary(middle, found_const);
            transform_ternary(right, found_const);
        }
        Expression::Array { expressions } => {
            for expr in expressions {
                transform_ternary(expr, found_const);
            }
        }
        Expression::Call { name, params } if (name == TERNARY_IF_THEN) => {
            if let [left, middle, right] = params.as_slice() {
                *found_const = true;
                *expression = Expression::Ternary {
                    left: Box::new(left.clone()),
                    middle: Box::new(middle.clone()),
                    right: Box::new(right.clone()),
                    operator: Operator::TernaryCondition,
                }
            } else {
                for expr in params {
                    transform_ternary(expr, found_const);
                }
            }
        }
        Expression::Call { name: _, params } => {
            for expr in params {
                transform_ternary(expr, found_const);
            }
        }
        _ => (),
    }
}

fn expressions_are_const(expressions: &[Expression]) -> bool {
    expressions
        .iter()
        .all(|e| matches!(e, Expression::Literal { value: _ }))
}

/// Evaluates [`Expression::Unary`], [`Expression::Binary`] [`Expression::Array`] into a single
/// [`Expression::Literal`] if all arguments are also an [`Expression::Literal`].
///
/// Evaluates [`Operator::TernaryCondition`] [`Expression::Ternary`] into either
/// the second or third argument, if the first argument is a [`Expression::Literal`].
///
/// Evaluates [`Expression::Call`] into a single [`Expression::Literal`] if all parameters
/// are [`Expression::Literal`] and the function is a pure function.
///
/// # Errors
///
/// Will return [`crate::Error`] if constant evaluation is not possible.
pub fn fold_constants(
    env: &dyn Environment,
    expression: &mut Expression,
    found_const: &mut bool,
) -> Result<()> {
    match expression {
        Expression::Unary { right, operator: _ } => match right.as_ref() {
            Expression::Literal { value: _ } => {
                *found_const = true;
                *expression = Expression::Literal {
                    value: execute(env, expression)?,
                }
            }
            _ => fold_constants(env, right, found_const)?,
        },
        Expression::Binary {
            left,
            right,
            operator: _,
        } => {
            if let (Expression::Literal { value: _ }, Expression::Literal { value: _ }) =
                (left.as_ref(), right.as_ref())
            {
                *found_const = true;
                *expression = Expression::Literal {
                    value: execute(env, expression)?,
                };
            } else {
                fold_constants(env, left, found_const)?;
                fold_constants(env, right, found_const)?;
            }
        }
        Expression::Ternary {
            left,
            middle,
            right,
            operator,
        } => {
            if let (Expression::Literal { value: left }, Operator::TernaryCondition) =
                (left.as_ref(), operator)
            {
                *found_const = true;
                if left.as_bool() {
                    *expression = *middle.clone();
                } else {
                    *expression = *right.clone();
                }
            } else {
                fold_constants(env, left, found_const)?;
                fold_constants(env, middle, found_const)?;
                fold_constants(env, right, found_const)?;
            }
        }
        Expression::Array { expressions } if expressions_are_const(expressions) => {
            *found_const = true;
            *expression = Expression::Literal {
                value: execute(env, expression)?,
            };
        }
        Expression::Array { expressions } => {
            for expr in expressions {
                fold_constants(env, expr, found_const)?;
            }
        }

        Expression::Call { name, params } if expressions_are_const(params) => {
            match env.function_exists(name, params.len()) {
                // only inline pure functions
                FunctionResult::Exists { pure } if pure => {
                    *found_const = true;
                    *expression = Expression::Literal {
                        value: execute(env, expression)?,
                    };
                }
                _ => (),
            }
        }
        Expression::Call { name: _, params } => {
            for expr in params {
                fold_constants(env, expr, found_const)?;
            }
        }
        _ => (),
    };

    Ok(())
}

/// Transforms an [`Expression`] tree by applying [`transform_ternary`] and
/// [`fold_constants`] in a loop until no further optimization is possible.
///
/// # Errors
///
/// Will return [`crate::Error`] if constant evaluation is not possible.
pub fn optimize(env: &dyn Environment, expression: &mut Expression) -> Result<()> {
    let mut found_const = false;

    loop {
        transform_ternary(expression, &mut found_const);
        fold_constants(env, expression, &mut found_const)?;

        if found_const {
            found_const = false; // repeat until no further folding is possible
        } else {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod test {

    use super::{optimize, transform_ternary};
    use crate::stdlib::common::TERNARY_IF_THEN;
    use crate::stdlib::extend_environment;
    use crate::{Expression, Operator, StaticEnvironment, Value};

    #[test]
    fn ternary_flat() {
        let mut expr = Expression::Call {
            name: String::from(TERNARY_IF_THEN),
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

        transform_ternary(&mut expr, &mut false);

        assert_eq!(ternary, expr);
    }

    #[test]
    fn ternary_nested() {
        let mut expr = Expression::Unary {
            right: Box::new(Expression::Call {
                name: String::from(TERNARY_IF_THEN),
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

        transform_ternary(&mut expr, &mut false);

        assert_eq!(ternary, expr);
    }

    #[test]
    fn fold_const_flat_binary() {
        let mut expr = Expression::Binary {
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

        optimize(&StaticEnvironment::default(), &mut expr).unwrap();
        assert_eq!(value, expr);
    }

    #[test]
    fn fold_const_flat_unary() {
        let mut expr = Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Number(5.0),
            }),
            operator: Operator::Minus,
        };

        let value = Expression::Literal {
            value: Value::Number(-5.0),
        };
        optimize(&StaticEnvironment::default(), &mut expr).unwrap();
        assert_eq!(value, expr);

        let mut expr = Expression::Unary {
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

        optimize(&StaticEnvironment::default(), &mut expr).unwrap();
        assert_eq!(value, expr);
    }

    #[test]
    fn fold_const_ternary() {
        let mut expr = Expression::Ternary {
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

        optimize(&StaticEnvironment::default(), &mut expr).unwrap();
        assert_eq!(value, expr);
    }

    #[test]
    fn fold_vectors() {
        let mut expr = Expression::Array {
            expressions: vec![Expression::Unary {
                right: Box::new(Expression::Call {
                    name: String::from(TERNARY_IF_THEN),
                    params: vec![
                        Expression::Literal {
                            value: Value::Boolean(true),
                        },
                        Expression::Call {
                            name: String::from(TERNARY_IF_THEN),
                            params: vec![
                                Expression::Literal {
                                    value: Value::Boolean(true),
                                },
                                Expression::Literal {
                                    value: Value::Number(3.0),
                                },
                            ],
                        },
                        Expression::Literal {
                            value: Value::Number(2.0),
                        },
                    ],
                }),
                operator: Operator::Minus,
            }],
        };

        let value = Expression::Array {
            expressions: vec![Expression::Unary {
                right: Box::new(Expression::Call {
                    name: String::from(TERNARY_IF_THEN),
                    params: vec![
                        Expression::Literal {
                            value: Value::Boolean(true),
                        },
                        Expression::Literal {
                            value: Value::Number(3.0),
                        },
                    ],
                }),
                operator: Operator::Minus,
            }],
        };
        optimize(&StaticEnvironment::default(), &mut expr).unwrap();

        assert_eq!(value, expr);
    }

    #[test]
    fn fold_array() {
        let mut expr = Expression::Array {
            expressions: vec![
                Expression::Literal {
                    value: Value::Boolean(true),
                },
                Expression::Literal {
                    value: Value::Boolean(false),
                },
            ],
        };

        let value = Expression::Literal {
            value: Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]),
        };

        optimize(&StaticEnvironment::default(), &mut expr).unwrap();

        assert_eq!(value, expr);
    }

    #[test]
    fn fold_pure_function() {
        let mut expr = Expression::Call {
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

        let value = Expression::Literal {
            value: Value::Number(20.0),
        };

        let mut env = StaticEnvironment::default();
        extend_environment(&mut env);

        optimize(&env, &mut expr).unwrap();

        assert_eq!(value, expr);
    }
}
