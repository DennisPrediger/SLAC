//! Functions to perform calculations with [`Value::Number`] variables.

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hasher},
};

use super::{
    default_number,
    error::{NativeError, NativeResult},
};

use crate::{
    environment::{Arity, Function},
    Value,
};

/// Returns all math functions.
#[rustfmt::skip]
pub fn functions() -> Vec<Function> {
    vec![
        Function::new(abs, Arity::required(1), "abs(value: Number): Number"),
        Function::new(arc_tan, Arity::required(1), "arc_tan(value: Number): Number"),
        Function::new(cos, Arity::required(1), "cos(value: Number): Number"),
        Function::new(exp, Arity::required(1), "exp(value: Number): Number"),
        Function::new(frac, Arity::required(1), "frac(value: Number): Number"),
        Function::new(ln, Arity::required(1), "ln(value: Number): Number"),
        Function::new(round, Arity::required(1), "round(value: Number): Number"),
        Function::new(sin, Arity::required(1), "sin(value: Number): Number"),
        Function::new(sqrt, Arity::required(1), "sqrt(value: Number): Number"),
        Function::new(trunc, Arity::required(1), "trunc(value: Number): Number"),
        Function::new(int_to_hex, Arity::required(1), "int_to_hex(value: Number): String"),
        Function::new(even, Arity::required(1), "even(value: Number): Boolean"),
        Function::new(odd, Arity::required(1), "odd(value: Number): Boolean"),
        Function::new(pow, Arity::optional(1, 1), "pow(value: Number, exponent: Number = 2): Number"),
        Function::new(random, Arity::optional(0, 1), "random(range: Number = 1): Number"),
    ]
}

macro_rules! generate_std_math_functions {
    ($($func_name:ident $std_func:ident),*) => {$(

        /// See the corresponding function descriptions in [`std::primitive::f64`].
        ///
        /// # Errors
        ///
        /// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
        /// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
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
/// * Declaration: `int_to_hex(value: Number): String`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn int_to_hex(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::String(format!("{:X}", value.trunc() as i64))),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Checks if a [`Value::Number`] is even and returns a [`Value::Boolean`].
///
/// * Declaration: `even(value: Number): Boolean`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn even(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::Boolean((*value as usize) % 2 == 0)),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Checks if a [`Value::Number`] is odd and returns a [`Value::Boolean`].
///
/// * Declaration: `odd(value: Number): Boolean`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn odd(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] => Ok(Value::Boolean((*value as usize) % 2 != 0)),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Raises a [`Value::Number`] to the power of an exponent.
///
/// * Declaration: `pow(value: Number, exponent: Number = 2): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn pow(params: &[Value]) -> NativeResult {
    let exponent = default_number(params, 1, 2.0)?;

    match params {
        [Value::Number(base), ..] => Ok(Value::Number(base.powf(exponent))),
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Generates a random-ish [`Value::Number`]. Uses [`RandomState`] and is very
/// much **not** cryptographicly secure
///
/// * Declaration: `random(range: Number = 1): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn random(params: &[Value]) -> NativeResult {
    let range = default_number(params, 0, 1.0)?;

    let random = RandomState::new().build_hasher().finish();
    Ok(Value::Number((random as f64 / u64::MAX as f64) * range))
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
            assert!(random(&vec![Value::Number(10000.0)]).unwrap() <= Value::Number(10000.0));
            assert!(random(&vec![Value::Number(-100.0)]).unwrap() >= Value::Number(-100.0));
            assert!(random(&vec![Value::Number(-100.0)]).unwrap() < Value::Number(0.0));
        }
    }
}
