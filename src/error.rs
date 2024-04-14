use std::result;

use thiserror::Error;

use crate::operator::Operator;
use crate::stdlib::NativeError;
use crate::token::Token;

/// The error type for failures while scanning, compiling or validation slac
/// expressions.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("unexpected end of file")]
    Eof,
    // scanner errors
    #[error("\"{0}\" is not a valid character")]
    InvalidCharacter(char),
    #[error("\"{0}\" is not a valid number")]
    InvalidNumber(String),
    #[error("unterminated string literal")]
    UnterminatedStringLiteral,
    #[error("encountered multiple expressions at Token \"{0:?}\"")]
    // compiler errors
    MultipleExpressions(Token),
    #[error("\"{0:?}\" is not a valid prefix Token")]
    NoValidPrefixToken(Token),
    #[error("\"{0:?}\" is not a valid infix Token")]
    NoValidInfixToken(Token),
    #[error("\"{0:?}\" is not a valid call target")]
    CallNotOnVariable(Token),
    #[error("previous Token not found")]
    PreviousTokenNotFound,
    #[error("invalid Token \"{0:?}\"")]
    InvalidToken(Token),
    #[error("\"{0:?}\" is not a valid Operator")]
    TokenNotAnOperator(Token),
    #[error("missing variable \"{0}\"")]
    // validation errors
    MissingVariable(String),
    #[error("missing function \"{0}\"")]
    MissingFunction(String),
    #[error("expected {1} parameters but got {2} for function \"{0}\"")]
    ParamCountMismatch(String, usize, usize), // name, expected, found
    #[error("invalid unary operator \"{0:?}\"")]
    InvalidUnaryOperator(Operator),
    #[error("invalid binary operator \"{0:?}\"")]
    InvalidBinaryOperator(Operator),
    #[error("invalid ternary operator \"{0:?}\"")]
    InvalidTernaryOperator(Operator),
    #[error("top level expression does not return a boolean value")]
    LiteralNotBoolean,
    // runtime errors
    #[error("undefined variable \"{0}\"")]
    UndefinedVariable(String),
    #[error("native function \"{0}\" encountered an error: \"{1}\"")]
    NativeFunctionError(String, NativeError),
}

/// A specialized [`Result`] type for [`Errors`](enum@Error) during the scanning, compiling or
/// validation phase.
pub type Result<T> = result::Result<T, Error>;
