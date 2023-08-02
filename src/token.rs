use std::fmt::Display;

use serde::de::Visitor;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer};

use crate::value::Value;

/// A [`Token`] is the smallest logical unit evaluated by the compiler.
/// It containes either an operator or a literal value.
#[rustfmt::skip]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
  // Single-character tokens
  LeftParen, RightParen, 
  LeftBracket, RightBracket, 
  Plus, Minus, Star, Slash, 
  Comma,
  // One or two character tokens
  Greater, GreaterEqual,
  Less, LessEqual,
  // Equality
  Equal, NotEqual,
  // Keywords
  And, Or, Not, Div, Mod,
  // Literal Values
  Literal(Value),
  Identifier(String)
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Comma => write!(f, ","),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Div => write!(f, "div"),
            Token::Mod => write!(f, "mod"),
            Token::Literal(name) => write!(f, "{}", name),
            Token::Identifier(name) => write!(f, "{}", name),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
struct TokenVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for TokenVisitor {
    type Value = Token;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a Token String")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "(" => Ok(Token::LeftParen),
            ")" => Ok(Token::RightParen),
            "[" => Ok(Token::LeftBracket),
            "]" => Ok(Token::RightBracket),
            "+" => Ok(Token::Plus),
            "-" => Ok(Token::Minus),
            "*" => Ok(Token::Star),
            "/" => Ok(Token::Slash),
            "," => Ok(Token::Comma),
            ">" => Ok(Token::Greater),
            ">=" => Ok(Token::GreaterEqual),
            "<" => Ok(Token::Less),
            "<=" => Ok(Token::LessEqual),
            "=" => Ok(Token::Equal),
            "<>" => Ok(Token::NotEqual),
            "and" => Ok(Token::And),
            "or" => Ok(Token::Or),
            "not" => Ok(Token::Not),
            "div" => Ok(Token::Div),
            "mod" => Ok(Token::Mod),
            _ => Err(serde::de::Error::custom(format!("unknown token {}", v))),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TokenVisitor)
    }
}

/// The precedences used to order the operators evaluated in the
/// [Pratt-Parser](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing)
/// when building the [`Expression`](crate::ast::Expression) tree.
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Or,         // or
    And,        // and
    Equality,   // = <>
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // ()
    Primary,    // Literals
}

#[rustfmt::skip]
impl From<&Token> for Precedence {
    fn from(token: &Token) -> Self {
        match token {
            Token::Minus | Token::Plus => Precedence::Term,
            Token::Star | Token::Slash | Token::Div | Token::Mod => Precedence::Factor,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => Precedence::Comparison,
            Token::And => Precedence::And,
            Token::Or => Precedence::Or,
            Token::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

impl Precedence {
    pub fn next(self) -> Precedence {
        match self {
            Precedence::None => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}
