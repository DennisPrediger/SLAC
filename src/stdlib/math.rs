//! Functions to perform calculations with [`Value::Number`] variables.

use getrandom::{getrandom, Error};

use super::{
    default_number,
    error::{NativeError, NativeResult},
    smart_vec, usize_from_f64,
};

use crate::{
    function::{Arity, Function},
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
        Function::impure(random, Arity::optional(0, 1), "random(range: Number = 1): Number"),
        Function::impure(choice, Arity::Variadic, "choice(...): Any"),
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
#[allow(clippy::cast_possible_truncation)]
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
        [Value::Number(value)] => Ok(Value::Boolean(usize_from_f64(*value) % 2 == 0)),
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
        [Value::Number(value)] => Ok(Value::Boolean(usize_from_f64(*value) % 2 != 0)),
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

#[allow(clippy::cast_precision_loss)]
fn get_random_float(max: f64) -> Result<f64, Error> {
    if max == 0.0 {
        return Ok(0.0); // shortcut for empty range
    }

    // get random bytes from the OS
    let mut buffer = [0u8; 8];
    getrandom(&mut buffer)?;

    // constrain the values to a float range
    let random = u64::from_le_bytes(buffer) as f64;
    Ok((random * max) / u64::MAX as f64)
}

fn get_random_int(max: usize) -> Result<usize, Error> {
    if max == 0 {
        return Ok(0); // shortcut for empty range
    }

    // get random bytes from the OS
    let mut buffer = [0u8; 8];
    getrandom(&mut buffer)?;

    // constrain the values to an integer range via modulo
    let random = usize::from_le_bytes(buffer);
    Ok(random % max)
}

/// Generates a random [`Value::Number`] provided by the os system source via [`mod@getrandom`].
///
/// * Declaration: `random(range: Number = 1): Number`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn random(params: &[Value]) -> NativeResult {
    let range = default_number(params, 0, 1.0)?;
    let result = get_random_float(range).map_err(|e| NativeError::CustomError(e.to_string()))?;

    Ok(Value::Number(result))
}

/// Returns a random choice of one of the provided parameters.
/// If a [`Value::Array`] is provided as sole parameter, returns one random [`Value`]
///
/// * Declaration: `choice(...): Any`
///
/// # Remarks
///
/// Uses [`mod@getrandom`] as RNG source.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterType`] no parameters are provided.
pub fn choice(params: &[Value]) -> NativeResult {
    let choices = smart_vec(params);
    let index: usize =
        get_random_int(choices.len()).map_err(|e| NativeError::CustomError(e.to_string()))?;

    choices
        .get(index)
        .cloned()
        .ok_or(NativeError::WrongParameterType)
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
            assert_eq!(random(&vec![Value::Number(0.0)]), Ok(Value::Number(0.0)));
        }

        for _ in 0..1000 {
            assert!(random(&vec![Value::Number(-1.0)]).unwrap() <= Value::Number(0.0));
            assert!(random(&vec![Value::Number(100.0)]).unwrap() >= Value::Number(0.0));
        }
    }

    #[test]
    fn math_choice() {
        let input = &vec![
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Number(123.00),
            Value::String("Hello".to_string()),
            Value::String("World".to_string()),
        ];

        for _ in 0..1000 {
            let res = choice(&input).unwrap();

            assert!(input.contains(&res));
        }

        assert_eq!(choice(&vec![]), Err(NativeError::WrongParameterType));
    }
}
