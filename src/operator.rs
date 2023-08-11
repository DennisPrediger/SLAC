#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::token::Token;

/// A binary or arithemtic operator.
#[derive(Debug, PartialEq, PartialOrd, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[rustfmt::skip]
pub enum Operator {
    Plus, Minus, Multiply, Divide,
    Greater, GreaterEqual,
    Less, LessEqual,
    Equal, NotEqual,
    And, Or, Xor, Not, 
    Div, Mod,
}

impl TryFrom<&Token> for Operator {
    type Error = String;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(Operator::Plus),
            Token::Minus => Ok(Operator::Minus),
            Token::Star => Ok(Operator::Multiply),
            Token::Slash => Ok(Operator::Divide),
            Token::Greater => Ok(Operator::Greater),
            Token::GreaterEqual => Ok(Operator::GreaterEqual),
            Token::Less => Ok(Operator::Less),
            Token::LessEqual => Ok(Operator::LessEqual),
            Token::Equal => Ok(Operator::Equal),
            Token::NotEqual => Ok(Operator::NotEqual),
            Token::And => Ok(Operator::And),
            Token::Or => Ok(Operator::Or),
            Token::Xor => Ok(Operator::Xor),
            Token::Not => Ok(Operator::Not),
            Token::Div => Ok(Operator::Div),
            Token::Mod => Ok(Operator::Mod),
            _ => Err(format!("unknown Token {value:?}")),
        }
    }
}
