use std::error;
use std::fmt;

use crate::token::Token;

#[derive(Debug, PartialEq)]
pub struct SyntaxError(String, String);

impl error::Error for SyntaxError {}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Syntax error: \"{}\", {}", self.0, self.1)
    }
}

impl SyntaxError {
    pub fn new(token: &Token, message: &str) -> Self {
        Self(format!("{:?}", token), message.to_string())
    }
}
