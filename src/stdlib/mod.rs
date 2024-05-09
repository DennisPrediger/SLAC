//! The SLAC standard library features various functions which can be included into a [`StaticEnvironment`].

use crate::environment::Function;
use crate::{StaticEnvironment, Value};

#[doc(inline)]
pub use self::error::NativeError;
#[doc(inline)]
pub use self::error::NativeResult;

pub mod common;
pub mod error;
pub mod math;
#[cfg(feature = "regex")]
pub mod regex;
pub mod string;
#[cfg(feature = "chrono")]
pub mod time;

#[cfg(feature = "zero_based_strings")]
pub const STRING_OFFSET: f64 = 0.0;

#[cfg(not(feature = "zero_based_strings"))]
pub const STRING_OFFSET: f64 = 1.0;

/// A function pointer used to execute native Rust functions.
/// All parameters to the function are inside a single Vec<[`Value`]>.
pub type NativeFunction = fn(&[Value]) -> NativeResult;

#[must_use]
pub fn builtins() -> Vec<Function> {
    [
        common::functions(),
        math::functions(),
        string::functions(),
        #[cfg(feature = "chrono")]
        time::functions(),
        #[cfg(feature = "regex")]
        regex::functions(),
    ]
    .concat()
}

/// Extends a [`StaticEnvironment`] with all standard library functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_functions(builtins());
}

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
        Some(Value::Number(value)) => Ok(*value),
        Some(_) => Err(NativeError::WrongParameterType),
        _ => Ok(default),
    }
}

pub(crate) fn get_index(index: &f64) -> Result<usize, NativeError> {
    if index >= &0.0 {
        Ok(*index as usize)
    } else {
        Err(NativeError::IndexNegative)
    }
}

pub(crate) fn get_string_index(index: &f64) -> Result<usize, NativeError> {
    get_index(index).map(|index| index - STRING_OFFSET as usize)
}

/// Returns the first parameter if it's an [`Value::Array`] or return all
/// parameters as varadic function.
pub(crate) fn smart_vec(params: &[Value]) -> &[Value] {
    match params {
        [Value::Array(v)] if (params.len() == 1) => v, // only one Array parameter
        _ => params,                                   // all varadic params
    }
}
