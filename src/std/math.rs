//! Functions to perform calculations with [`Value::Number`] variables.

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

use super::error::{NativeError, NativeResult};
use crate::{StaticEnvironment, Value};

/// Extends a [`StaticEnvironment`] with functions to manipulate [`Value::Number`]
/// variables and various common mathematical constants.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("abs", Some(1), 0, abs);
    env.add_native_func("arc_tan", Some(1), 0, arc_tan);
    env.add_native_func("cos", Some(1), 0, cos);
    env.add_native_func("exp", Some(1), 0, exp);
    env.add_native_func("frac", Some(1), 0, frac);
    env.add_native_func("round", Some(1), 0, round);
    env.add_native_func("ln", Some(1), 0, ln);
    env.add_native_func("sin", Some(1), 0, sin);
    env.add_native_func("sqrt", Some(1), 0, sqrt);
    env.add_native_func("trunc", Some(1), 0, trunc);

    env.add_native_func("int_to_hex", Some(1), 0, int_to_hex);
    env.add_native_func("even", Some(1), 0, even);
    env.add_native_func("odd", Some(1), 0, odd);
    env.add_native_func("pow", Some(2), 1, pow);
    env.add_native_func("random", Some(1), 1, random);
}

macro_rules! generate_std_math_functions {
    ($($func_name:ident $std_func:ident),*) => {$(

        /// See the corresponding function descriptions in [`std::primitive::f64`].
        ///
        /// # Errors
        ///
        /// Returns an error if there is no parameter supplied or the parameter
        /// is not a [`Value::Number`].
        pub fn $func_name(params: &[Value]) -> NativeResult {
            match params {
                [Value::Number(value)] => Ok(Value::Number(value.$std_func())),
                [_] => Err(NativeError::WrongParameterType),
                _ => Err(NativeError::WrongParameterCount(1)),
            }
        }

    )*};
}

// Generate common parameter-less f64 functions.
generate_std_math_functions!(
    abs abs,
    arc_tan atan,
    cos cos,
    exp exp,
    frac fract,
    ln ln,
    round round,
    sin sin,
    sqrt sqrt,
    trunc trunc
);

/// Converts a [`Value::Number`] to an uppercase hex [`Value::String`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn int_to_hex(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::String(format!("{:X}", value.trunc() as i64))),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Checks if a [`Value::Number`] is even and returns a [`Value::Boolean`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn even(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::Boolean((*value as usize) % 2 == 0)),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Checks if a [`Value::Number`] is odd and returns a [`Value::Boolean`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn odd(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::Boolean((*value as usize) % 2 != 0)),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Raises a [`Value::Number`] to the power in the second [`Value::Number`] parameter.
///
/// # Remark
///
/// The second parameter is optional and defaults to 2.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn pow(params: &[Value]) -> NativeResult {
    let exponent = match params.get(1) {
        Some(Value::Number(exponent)) => *exponent,
        Some(_) => return Err(NativeError::WrongParameterType),
        _ => 2.0,
    };

    match params {
        [Value::Number(base), ..] => Ok(Value::Number(base.powf(exponent))),
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Generates a random-ish [`Value::Number`]. Uses [`RandomState`] and is very
/// much **not** cryptographicly secure
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn random(params: &[Value]) -> NativeResult {
    match params.get(0).unwrap_or(&Value::Number(1.0)) {
        Value::Number(range) => {
            let random = RandomState::new().build_hasher().finish();
            Ok(Value::Number((random as f64 / u64::MAX as f64) * range))
        }
        _ => Err(NativeError::WrongParameterType),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;

    #[test]
    fn math_abs() {
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(10.0)]));
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(-10.0)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(12.34)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(-12.34)]));

        assert!(abs(&vec![Value::String(String::from("-12.34"))]).is_err());
    }

    #[test]
    fn math_int_to_hex() {
        assert_eq!(
            Ok(Value::String(String::from("3039"))),
            int_to_hex(&vec![Value::Number(12345.0)])
        );
        assert_eq!(
            Ok(Value::String(String::from("DEADBEEF"))),
            int_to_hex(&vec![Value::Number(3735928559.0)])
        );
        assert_eq!(
            Ok(Value::String(String::from("DEADBEEF"))),
            int_to_hex(&vec![Value::Number(3735928559.1234)])
        );
    }

    #[test]
    fn math_even() {
        assert_eq!(Ok(Value::Boolean(true)), even(&vec![Value::Number(10.0)]));
        assert_eq!(Ok(Value::Boolean(false)), even(&vec![Value::Number(11.0)]));
        assert_eq!(Ok(Value::Boolean(true)), even(&vec![Value::Number(0.0)]));
        assert_eq!(Ok(Value::Boolean(false)), even(&vec![Value::Number(1.0)]));
        assert_eq!(Ok(Value::Boolean(true)), even(&vec![Value::Number(2.0)]));
    }

    #[test]
    fn math_odd() {
        assert_eq!(Ok(Value::Boolean(false)), odd(&vec![Value::Number(10.0)]));
        assert_eq!(Ok(Value::Boolean(true)), odd(&vec![Value::Number(11.0)]));
        assert_eq!(Ok(Value::Boolean(false)), odd(&vec![Value::Number(0.0)]));
        assert_eq!(Ok(Value::Boolean(true)), odd(&vec![Value::Number(1.0)]));
        assert_eq!(Ok(Value::Boolean(false)), odd(&vec![Value::Number(2.0)]));
    }

    #[test]
    fn math_odd_even() {
        for i in -1000..1000 {
            assert_ne!(
                even(&vec![Value::Number(i as f64)]),
                odd(&vec![Value::Number(i as f64)])
            );
        }
    }

    #[test]
    fn math_pow() {
        assert_eq!(
            Value::Number(100.0),
            pow(&vec![Value::Number(10.0)]).unwrap()
        );

        assert_eq!(
            Value::Number(0.001),
            pow(&vec![Value::Number(10.0), Value::Number(-3.0)]).unwrap()
        );

        assert!(pow(&vec![]).is_err());
        assert!(pow(&vec![Value::Boolean(true)]).is_err());
        assert!(pow(&vec![Value::Number(10.0), Value::Boolean(true)]).is_err());
    }

    #[test]
    fn math_round() {
        assert_eq!(
            Value::Number(10.0),
            round(&vec![Value::Number(10.4)]).unwrap()
        );
        assert_eq!(
            Value::Number(11.0),
            round(&vec![Value::Number(10.5)]).unwrap()
        );
        assert_eq!(
            Value::Number(-10.0),
            round(&vec![Value::Number(-10.4)]).unwrap()
        );
        assert_eq!(
            Value::Number(-11.0),
            round(&vec![Value::Number(-10.5)]).unwrap()
        );

        assert!(round(&vec![]).is_err());
    }

    #[test]
    fn math_random() {
        for _ in 0..1000 {
            assert!(random(&vec![]).unwrap() <= Value::Number(1.0));
            assert!(random(&vec![]).unwrap() > Value::Number(0.0));
            assert!(random(&vec![Value::Number(10000.0)]) <= Ok(Value::Number(10000.0)));
            assert!(random(&vec![Value::Number(-100.0)]) >= Ok(Value::Number(-100.0)));
            assert!(random(&vec![Value::Number(-100.0)]) < Ok(Value::Number(0.0)));
        }
    }
}
