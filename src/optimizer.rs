use crate::{Expression, Operator};

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

#[cfg(test)]
mod test {
    use super::transform_ternary;
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
}
