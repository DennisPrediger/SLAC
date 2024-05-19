use thiserror::Error;

use crate::Value;

/// Error types created by [`super::NativeFunction`] calls.
/// `NativeError::CustomError` can be used for general purpose errors.
#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug, PartialEq)]
pub enum NativeError {
    #[error("function \"{0}\" not found")]
    FunctionNotFound(String),
    #[error("not enough parameters: \"{0}\" expected")]
    WrongParameterCount(usize),
    #[error("wrong parameter type")]
    WrongParameterType,
    #[error("index \"{0}\" is out of bounds")]
    IndexOutOfBounds(usize),
    #[error("index must not be negative")]
    IndexNegative,
    #[error("{0}")]
    CustomError(String),
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
