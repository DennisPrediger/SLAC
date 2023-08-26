use std::error;
use std::fmt;
use std::fmt::Display;
use std::result;

use crate::operator::Operator;
use crate::token::Token;

/// The error type for failures while scanning, compiling or validation slac
/// expressions.
#[derive(Debug, PartialEq)]
pub enum Error {
    Eof,
    // scanner errors
    InvalidCharacter(char),
    InvalidNumber(String),
    UnterminatedStringLiteral,
    // compiler errors
    MultipleExpressions(Token),
    NoValidPrefixToken(Token),
    NoValidInfixToken(Token),
    CallNotOnVariable(Token),
    PreviousTokenNotFound,
    InvalidToken(Token),
    TokenNotAnOperator(Token),
    // validation errors
    MissingVariable(String),
    MissingFunction(String),
    ParamCountMismatch(String, usize, usize), // name, expected, found
    InvalidUnaryOperator(Operator),
    InvalidBinaryOperator(Operator),
    LiteralNotBoolean,
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Eof => write!(f, "unexpected end of file"),
            Error::InvalidCharacter(char) => write!(f, "\"{char}\" is not a valid character"),
            Error::InvalidNumber(number) => write!(f, "\"{number}\" is not a valid number"),
            Error::UnterminatedStringLiteral => write!(f, "unterminated string literal"),
            Error::MultipleExpressions(token) => {
                write!(f, "encountered multiple expressions at token \"{token:?}\"")
            }
            Error::NoValidPrefixToken(token) => {
                write!(f, "\"{token:?}\" is not a valid prefix token")
            }
            Error::NoValidInfixToken(token) => {
                write!(f, "\"{token:?}\" is not a valid infix token")
            }
            Error::CallNotOnVariable(token) => {
                write!(f, "\"{token:?}\" is not a valid call target")
            }
            Error::PreviousTokenNotFound => write!(f, "previous token not found"),
            Error::InvalidToken(token) => write!(f, "invalid token \"{token:?}\""),
            Error::TokenNotAnOperator(token) => {
                write!(f, "\"{token:?}\" is not a valid operator")
            }
            Error::MissingVariable(name) => writeln!(f, "missing variable \"{name}\""),
            Error::MissingFunction(name) => writeln!(f, "missing function \"{name}\""),
            Error::ParamCountMismatch(name, expected, found) => writeln!(
                f,
                "expected {expected} parameters but got {found} for function \"{name}\""
            ),
            Error::InvalidUnaryOperator(operator) => {
                writeln!(f, "invalid unary operator \"{operator:?}\"")
            }
            Error::InvalidBinaryOperator(operator) => {
                writeln!(f, "invalid binary operator \"{operator:?}\"")
            }
            Error::LiteralNotBoolean => {
                writeln!(f, "top level expression does not return a boolean value")
            }
        }
    }
}

/// A specialized [`Result`] type for errors during the scanning, compiling or
/// validation phase.
pub type Result<T> = result::Result<T, Error>;
