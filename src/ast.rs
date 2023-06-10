use crate::token::Token;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Expression {
    Unary {
        right: Box<Expression>,
        operator: Token,
    },
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: Token,
    },
    Literal(Token),
    Variable(Token),
    Error(String),
}
