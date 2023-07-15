use slac::{ast::Expression, compile, token::Token};

#[test]
fn test_single_boolean_true() {
    let result = compile("True");
    let expected = Expression::Literal(Token::Boolean(true));

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_single_boolean_false() {
    let result = compile("False");
    let expected = Expression::Literal(Token::Boolean(false));

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_single_variable() {
    let result = compile("SOME_VAR");
    let expected = Expression::Variable(Token::Identifier("SOME_VAR".to_string()));

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_simple_addition() {
    let result = compile("1 + 2");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Number(1.0))),
        right: Box::new(Expression::Literal(Token::Number(2.0))),
        operator: Token::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_multiply_addition() {
    let result = compile("1 * 2 + 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(1.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Star,
        }),
        right: Box::new(Expression::Literal(Token::Number(3.0))),
        operator: Token::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_addition_multiply() {
    let result = compile("1 + 2 * 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Number(1.0))),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(2.0))),
            right: Box::new(Expression::Literal(Token::Number(3.0))),
            operator: Token::Star,
        }),
        operator: Token::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_group_addition_multiply() {
    let result = compile("(1 + 2) * 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(1.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Plus,
        }),
        right: Box::new(Expression::Literal(Token::Number(3.0))),
        operator: Token::Star,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_and() {
    let result = compile("True and False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Boolean(true))),
        right: Box::new(Expression::Literal(Token::Boolean(false))),
        operator: Token::And,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_or() {
    let result = compile("True or False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Boolean(true))),
        right: Box::new(Expression::Literal(Token::Boolean(false))),
        operator: Token::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_and_or() {
    let result = compile("False and True or False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Boolean(false))),
            right: Box::new(Expression::Literal(Token::Boolean(true))),
            operator: Token::And,
        }),
        right: Box::new(Expression::Literal(Token::Boolean(false))),
        operator: Token::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_or_and() {
    let result = compile("False or True and False");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Boolean(false))),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Boolean(true))),
            right: Box::new(Expression::Literal(Token::Boolean(false))),
            operator: Token::And,
        }),
        operator: Token::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_unary_not() {
    let result = compile("not False");
    let expected = Expression::Unary {
        right: Box::new(Expression::Literal(Token::Boolean(false))),
        operator: Token::Not,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_unary_not_and() {
    let result = compile("not False or True");
    let expected = Expression::Binary {
        left: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal(Token::Boolean(false))),
            operator: Token::Not,
        }),
        right: Box::new(Expression::Literal(Token::Boolean(true))),
        operator: Token::Or,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_equals() {
    let result = compile("1 = 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Number(1.0))),
        right: Box::new(Expression::Literal(Token::Number(3.0))),
        operator: Token::Equal,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_not_equals() {
    let result = compile("1 <> 3");
    let expected = Expression::Binary {
        left: Box::new(Expression::Literal(Token::Number(1.0))),
        right: Box::new(Expression::Literal(Token::Number(3.0))),
        operator: Token::NotEqual,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_not_equals_unary() {
    let result = compile("not true <> not false");
    let expected = Expression::Binary {
        left: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal(Token::Boolean(true))),
            operator: Token::Not,
        }),
        right: Box::new(Expression::Unary {
            right: Box::new(Expression::Literal(Token::Boolean(false))),
            operator: Token::Not,
        }),
        operator: Token::NotEqual,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_add_equals() {
    let result = compile("1 + 2 = 10 - 7");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(1.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Plus,
        }),
        right: Box::new(Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(10.0))),
            right: Box::new(Expression::Literal(Token::Number(7.0))),
            operator: Token::Minus,
        }),
        operator: Token::Equal,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_add_add_add() {
    let result = compile("1 + 2 + 3 + 4 + 5");

    let expected = Expression::Binary {
        left: Box::new(Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal(Token::Number(1.0))),
                    right: Box::new(Expression::Literal(Token::Number(2.0))),
                    operator: Token::Plus,
                }),
                right: Box::new(Expression::Literal(Token::Number(3.0))),
                operator: Token::Plus,
            }),
            right: Box::new(Expression::Literal(Token::Number(4.0))),
            operator: Token::Plus,
        }),
        right: Box::new(Expression::Literal(Token::Number(5.0))),
        operator: Token::Plus,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_function_call() {
    let result = compile("max(1 + 5, 3) > 2");

    let expected = Expression::Binary {
        left: Box::new(Expression::Call(
            Token::Identifier("max".to_string()),
            vec![
                Expression::Binary {
                    left: Box::new(Expression::Literal(Token::Number(1.0))),
                    right: Box::new(Expression::Literal(Token::Number(5.0))),
                    operator: Token::Plus,
                },
                Expression::Literal(Token::Number(3.0)),
            ],
        )),
        right: Box::new(Expression::Literal(Token::Number(2.0))),
        operator: Token::Greater,
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_function_call_no_params() {
    let result = compile("Now() > current_date");

    let expected = Expression::Binary {
        left: Box::new(Expression::Call(
            Token::Identifier("Now".to_string()),
            vec![],
        )),
        right: Box::new(Expression::Variable(Token::Identifier(
            "current_date".to_string(),
        ))),
        operator: Token::Greater,
    };

    assert_eq!(result, Ok(expected));
}