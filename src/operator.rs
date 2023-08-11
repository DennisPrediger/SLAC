use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::token::Token;

/// A binary or arithemtic operator.
#[derive(Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[rustfmt::skip]
pub enum Operator {
    Plus, Minus,  Star, Slash,
    Greater, GreaterEqual,
    Less, LessEqual,
    Equal, NotEqual,
    And, Or, Xor, Not, 
    Div, Mod,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Star => write!(f, "*"),
            Operator::Slash => write!(f, "/"),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterEqual => write!(f, ">="),
            Operator::Less => write!(f, "<"),
            Operator::LessEqual => write!(f, "<="),
            Operator::Equal => write!(f, "="),
            Operator::NotEqual => write!(f, "<>"),
            Operator::And => write!(f, "and"),
            Operator::Or => write!(f, "or"),
            Operator::Xor => write!(f, "xor"),
            Operator::Not => write!(f, "not"),
            Operator::Div => write!(f, "div"),
            Operator::Mod => write!(f, "mod"),
        }
    }
}

impl TryFrom<&Token> for Operator {
    type Error = String;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(Operator::Plus),
            Token::Minus => Ok(Operator::Minus),
            Token::Star => Ok(Operator::Star),
            Token::Slash => Ok(Operator::Slash),
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
