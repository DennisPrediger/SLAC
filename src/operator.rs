use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Serialize, Serializer};

use crate::token::Token;

/// A binary or arithemtic operator.
#[derive(Debug, PartialEq, PartialOrd)]
#[rustfmt::skip]
pub enum Operator {
    Plus, Minus,  Star, Slash,
    Greater, GreaterEqual,
    Less, LessEqual,
    Equal, NotEqual,
    And, Or, Not, 
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
            Token::Not => Ok(Operator::Not),
            Token::Div => Ok(Operator::Div),
            Token::Mod => Ok(Operator::Mod),
            _ => Err(format!("unknown Token {:?}", value)),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Operator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Operator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(OperatorVisitor)
    }
}

#[cfg(feature = "serde")]
struct OperatorVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for OperatorVisitor {
    type Value = Operator;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a Operator String")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "+" => Ok(Operator::Plus),
            "-" => Ok(Operator::Minus),
            "*" => Ok(Operator::Star),
            "/" => Ok(Operator::Slash),
            ">" => Ok(Operator::Greater),
            ">=" => Ok(Operator::GreaterEqual),
            "<" => Ok(Operator::Less),
            "<=" => Ok(Operator::LessEqual),
            "=" => Ok(Operator::Equal),
            "<>" => Ok(Operator::NotEqual),
            "and" => Ok(Operator::And),
            "or" => Ok(Operator::Or),
            "not" => Ok(Operator::Not),
            "div" => Ok(Operator::Div),
            "mod" => Ok(Operator::Mod),
            _ => Err(serde::de::Error::custom(format!("unknown Operator {}", v))),
        }
    }
}
