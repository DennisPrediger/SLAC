use std::{collections::hash_map::RandomState, hash::{BuildHasher, Hasher}};

use crate::{StaticEnvironment, Value};

pub const PI: f64 = std::f64::consts::PI;
pub const E: f64 = std::f64::consts::E;
pub const TAU: f64 = std::f64::consts::TAU;

pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_var("pi", Value::Number(PI));
    env.add_var("e", Value::Number(E));
    env.add_var("tau", Value::Number(TAU));

    env.add_native_func("arc_tan", Some(1), arc_tan);
    env.add_native_func("cos", Some(1), cos);
    env.add_native_func("exp", Some(1), exp);
    env.add_native_func("frac", Some(1), frac);
    env.add_native_func("ln", Some(1), ln);
    env.add_native_func("sin", Some(1), sin);
    env.add_native_func("sqrt", Some(1), sqrt);

    env.add_native_func("abs", Some(1), abs);
    env.add_native_func("even", Some(1), even);
    env.add_native_func("odd", Some(1), odd);
    env.add_native_func("pow", None, pow);
    env.add_native_func("random", Some(1), random);
    env.add_native_func("round", Some(1), round);
}

macro_rules! generate_std_math_function {
    ($($func_name:ident $std_func:ident),*) => {$(
        /// See [std::primitive::f64].
        pub fn $func_name(params: &[Value]) -> Result<Value, String> {
            match params.get(0) {
                Some(Value::Number(value)) => Ok(Value::Number(value.$std_func())),
                Some(_) => Err(String::from("wrong parameter type")),
                None => Err(String::from("not enough Parameters")),
            }
        }
    )*};
}

generate_std_math_function!(abs abs,
                            arc_tan atan, 
                            cos cos, 
                            exp exp, 
                            frac fract, 
                            ln ln, 
                            sin sin, 
                            sqrt sqrt);

/// !!! todo
pub fn even(params: &[Value]) -> Result<Value, String> {
    match params.get(0) {
        Some(Value::Number(value)) => Ok(Value::Boolean((*value as usize) % 2 == 0)),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("not enough Parameters")),
    }
}

/// !!! todo
pub fn odd(params: &[Value]) -> Result<Value, String> {
    match params.get(0) {
        Some(Value::Number(value)) => Ok(Value::Boolean((*value as usize) % 2 != 0)),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("not enough Parameters")),
    }
}

/// Raises a [`Value::Number`] to a power in the second parameter.
///
/// # Remark
/// The second parameter is optional and defaults to 2.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied.
pub fn pow(params: &[Value]) -> Result<Value, String> {
    match (params.get(0), params.get(1)) {
        (Some(Value::Number(base)), exp) => {
            let exp = match exp {
                Some(Value::Number(exp)) => *exp,
                _ => 2.0,
            };
            Ok(Value::Number(base.powf(exp)))
        }
        (Some(_), _) => Err(String::from("wrong parameter type")),
        _ => Err("not enough parameters".to_string()),
    }
}

/// Generates a random-ish [`Value::Number`]. Uses [`RandomState`] and is very 
/// much **not** cryptographicly secure
/// # Errors
/// Will return an error if the optional Range parameter is not a [`Value::Number`].
pub fn random(params: &[Value]) -> Result<Value, String> {
    match params.get(0).unwrap_or(&Value::Number(1.0)) {
        Value::Number(range) => {
            let random = RandomState::new().build_hasher().finish();
            Ok(Value::Number((random as f64 / u64::MAX as f64) * range))
        },
        _=> Err(String::from("wrong parameter type"))
    }
}

/// Rounds a [`Value::Number`] to the nearest integer.
//////
/// # Errors
/// Will return an error if not at least one parameter is supplied.
pub fn round(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::Number(v)) => Ok(Value::Number(v.round())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("not enough Parameters")),
    }
}

#[cfg(test)]
mod test {
    use crate::Value;
    use super::{abs, pow,even, round, random, odd};

    #[test]
    fn math_abs() {
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(10.0)]));
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(-10.0)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(12.34)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(-12.34)]));

        assert!(abs(&vec![Value::String("-12.34".to_string())]).is_err());
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
            assert_ne!(even(&vec![Value::Number(i as f64)]), odd(&vec![Value::Number(i as f64)]));
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

        assert_eq!(
            Value::Number(100.0),
            pow(&vec![Value::Number(10.0), Value::Boolean(true)]).unwrap()
        );

        assert!(pow(&vec![]).is_err());
        assert!(pow(&vec![Value::Boolean(true)]).is_err());
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
