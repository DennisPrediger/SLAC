//! The SLAC standard library features various functions which can be included into a [`StaticEnvironment`].

use crate::{StaticEnvironment, Value};

#[doc(inline)]
pub use self::error::NativeError;
#[doc(inline)]
pub use self::error::NativeResult;

pub mod common;
pub mod error;
pub mod math;
pub mod string;
#[cfg(feature = "chrono")]
pub mod time;

/// A function pointer used to execute native Rust functions.
/// All parameters to the function are inside a single Vec<[`Value`]>.
pub type NativeFunction = fn(&[Value]) -> NativeResult;

/// Extends a [`StaticEnvironment`] with all standard library functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    common::extend_environment(env);
    math::extend_environment(env);
    string::extend_environment(env);

    #[cfg(feature = "chrono")]
    time::extend_environment(env);

pub(crate) fn default_string<'a>(
    params: &'a [Value],
    index: usize,
    default: &'a str,
) -> Result<&'a str, NativeError> {
    match params.get(index) {
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(NativeError::WrongParameterType),
        _ => Ok(default),
    }
}

pub(crate) fn default_number(
    params: &[Value],
    index: usize,
    default: f64,
) -> Result<f64, NativeError> {
    match params.get(index) {
        Some(Value::Number(value)) => Ok(value.clone()),
        Some(_) => Err(NativeError::WrongParameterType),
        _ => Ok(default),
    }
}
