use std::error;
use std::fmt;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct SyntaxError(pub String);

impl error::Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Syntax error: {}", self)
    }
}

impl From<&str> for SyntaxError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl SyntaxError {
    pub fn expected(expected: &str, recieved: &Token) -> Self {
        Self(format!("Expected {} got \"{:?}\"", expected, recieved))
    }
}

pub type Result<T> = std::result::Result<T, SyntaxError>;
