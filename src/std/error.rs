use std::{error, fmt::Display};

use crate::Value;

/// Error types created by [`super::NativeFunction`] calls.
/// `NativeError::CustomError` can be used for general purpose errors.
#[derive(Debug, PartialEq, PartialOrd)]
pub enum NativeError {
    WrongParameterCount(usize),
    WrongParameterType,
    IndexOutOfBounds(usize),
    CustomError(String),
}

impl error::Error for NativeError {}

impl Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeError::WrongParameterCount(count) => {
                write!(f, "not enough parameters: {count} expected")
            }
            NativeError::WrongParameterType => write!(f, "wrong parameter type"),
            NativeError::IndexOutOfBounds(index) => write!(f, "index {index} is out of bounds"),
            NativeError::CustomError(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<&str> for NativeError {
    fn from(value: &str) -> Self {
        Self::CustomError(value.to_string())
    }
}

impl From<String> for NativeError {
    fn from(value: String) -> Self {
        Self::CustomError(value)
    }
}

/// A specialized [`Result`] type for [`super::NativeFunction`] results.
pub type NativeResult = Result<Value, NativeError>;
