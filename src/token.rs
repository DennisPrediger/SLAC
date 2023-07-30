#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

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

#[cfg(feature = "serde")]
impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Token::LeftParen => serializer.serialize_char('('),
            Token::RightParen => serializer.serialize_char(')'),
            Token::LeftBracket => serializer.serialize_char('['),
            Token::RightBracket => serializer.serialize_char(']'),
            Token::Plus => serializer.serialize_char('+'),
            Token::Minus => serializer.serialize_char('-'),
            Token::Star => serializer.serialize_char('*'),
            Token::Slash => serializer.serialize_char('/'),
            Token::Comma => serializer.serialize_char(','),
            Token::Greater => serializer.serialize_char('>'),
            Token::GreaterEqual => serializer.serialize_str(">="),
            Token::Less => serializer.serialize_char('<'),
            Token::LessEqual => serializer.serialize_str("<="),
            Token::Equal => serializer.serialize_char('='),
            Token::NotEqual => serializer.serialize_str("<>"),
            Token::And => serializer.serialize_str("and"),
            Token::Or => serializer.serialize_str("or"),
            Token::Not => serializer.serialize_str("not"),
            Token::Div => serializer.serialize_str("div"),
            Token::Mod => serializer.serialize_str("mod"),
            Token::Identifier(name) => serializer.serialize_str(name),
            Token::Literal(value) => value.serialize(serializer),
        }
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
