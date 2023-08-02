use std::error;
use std::fmt;

use crate::token::Token;

/// A `SyntaxError` occures on invalid user input in the [scanner](crate::scanner) or
/// [compiler](crate::compiler) Phase.
#[derive(Debug, PartialEq)]
pub struct SyntaxError(pub String);

impl error::Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Syntax error: {}", self.0)
    }
}

impl From<&str> for SyntaxError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for SyntaxError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl SyntaxError {
    pub fn expected(expected: &str, recieved: &Token) -> Self {
        Self(format!("Expected {expected} got \"{recieved:?}\""))
    }
}

pub type Result<T> = std::result::Result<T, SyntaxError>;
