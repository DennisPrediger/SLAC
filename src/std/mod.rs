use crate::StaticEnvironment;

pub mod common;
pub mod math;
pub mod string;
#[cfg(feature = "chrono")]
pub mod time;

pub fn extend_environment(env: &mut StaticEnvironment) {
    common::extend_environment(env);
    math::extend_environment(env);
    string::extend_environment(env);

    #[cfg(feature = "chrono")]
    time::extend_environment(env);
}
