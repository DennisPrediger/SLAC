//! The SLAC standard library features various functions which can be included into a [`StaticEnvironment`].

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

/// Extends a [`StaticEnvironment`] with all standard library functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_functions(common::functions());
    env.add_functions(math::functions());
    env.add_functions(string::functions());
    #[cfg(feature = "chrono")]
    env.add_functions(time::functions());
    #[cfg(feature = "regex")]
    env.add_functions(regex::functions());
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
