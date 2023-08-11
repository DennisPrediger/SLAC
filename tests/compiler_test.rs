use slac::{ast::Expression, compile, operator::Operator, value::Value};

#[test]
fn single_boolean_true() {
    let result = compile("True");
    let expected = Expression::Literal {
        value: Value::Boolean(true),
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn single_boolean_false() {
    let result = compile("False");
    let expected = Expression::Literal {
        value: Value::Boolean(false),
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn single_variable() {
    let result = compile("SOME_VAR");
    let expected = Expression::Variable {
        name: "SOME_VAR".to_string(),
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn simple_addition() {
    let result = compile("1 + 2");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Number(1.0),
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(2.0),
        }),
        operator: Operator::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn multiply_addition() {
    let result = compile("1 * 2 + 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Multiply,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(3.0),
        }),
        operator: Operator::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn addition_multiply() {
    let result = compile("1 + 2 * 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Number(1.0),
        }),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            operator: Operator::Multiply,
        }),
        operator: Operator::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn group_addition_multiply() {
    let result = compile("(1 + 2) * 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Plus,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(3.0),
        }),
        operator: Operator::Multiply,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn and() {
    let result = compile("True and False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Boolean(true),
        }),
        right: Box::new(Expression::Literal {
            value: Value::Boolean(false),
        }),
        operator: Operator::And,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn or() {
    let result = compile("True or False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Boolean(true),
        }),
        right: Box::new(Expression::Literal {
            value: Value::Boolean(false),
        }),
        operator: Operator::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn and_or() {
    let result = compile("False and True or False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            operator: Operator::And,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Boolean(false),
        }),
        operator: Operator::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn or_and() {
    let result = compile("False or True and False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Boolean(false),
        }),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::And,
        }),
        operator: Operator::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn unary_not() {
    let result = compile("not False");
    let expected = Expression::Unary {
        right: Box::new(Expression::Literal {
            value: Value::Boolean(false),
        }),
        operator: Operator::Not,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn unary_not_and() {
    let result = compile("not False or True");
    let expected = Expression::Binary {
        left: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::Not,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Boolean(true),
        }),
        operator: Operator::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn equals() {
    let result = compile("1 = 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Number(1.0),
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(3.0),
        }),
        operator: Operator::Equal,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn not_equals() {
    let result = compile("1 <> 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal {
            value: Value::Number(1.0),
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(3.0),
        }),
        operator: Operator::NotEqual,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn not_equals_unary() {
    let result = compile("not true <> not false");
    let expected = Expression::Binary {
        left: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            operator: Operator::Not,
        }),
        right: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::Not,
        }),
        operator: Operator::NotEqual,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn add_equals() {
    let result = compile("1 + 2 = 10 - 7");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Plus,
        }),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(10.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(7.0),
            }),
            operator: Operator::Minus,
        }),
        operator: Operator::Equal,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn add_add_add() {
    let result = compile("1 + 2 + 3 + 4 + 5");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal {
                        value: Value::Number(1.0),
                    }),
                    right: Box::new(Expression::Literal {
                        value: Value::Number(2.0),
                    }),
                    operator: Operator::Plus,
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Number(3.0),
                }),
                operator: Operator::Plus,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(4.0),
            }),
            operator: Operator::Plus,
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(5.0),
        }),
        operator: Operator::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn function_call() {
    let result = compile("max(1 + 5, 3) > 2");

    let expected = Expression::Binary {
        left: Box::new(Expression::Call {
            name: "max".to_string(),
            params: vec![
                Expression::Binary {
                    left: Box::new(Expression::Literal {
                        value: Value::Number(1.0),
                    }),
                    right: Box::new(Expression::Literal {
                        value: Value::Number(5.0),
                    }),
                    operator: Operator::Plus,
                },
                Expression::Literal {
                    value: Value::Number(3.0),
                },
            ],
        }),
        right: Box::new(Expression::Literal {
            value: Value::Number(2.0),
        }),
        operator: Operator::Greater,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn function_call_no_params() {
    let result = compile("Now() > current_date");

    let expected = Expression::Binary {
        left: Box::new(Expression::Call {
            name: "Now".to_string(),
            params: vec![],
        }),
        right: Box::new(Expression::Variable {
            name: "current_date".to_string(),
        }),
        operator: Operator::Greater,
    };

    assert_eq!(result, Ok(expected));
}
