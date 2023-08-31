//! The SLAC standard library features various functions which can be included into a [`StaticEnvironment`].

use crate::StaticEnvironment;

pub mod common;
pub mod math;
pub mod string;
#[cfg(feature = "chrono")]
pub mod time;

/// Extends a [`StaticEnvironment`] with all standard library functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    common::extend_environment(env);
    math::extend_environment(env);
    string::extend_environment(env);

    #[cfg(feature = "chrono")]
    time::extend_environment(env);
}
