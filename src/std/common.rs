//! Common functions and constants for converting variables into different
//! [`Value`] types or check, extract and extend [`Value::Array`] variables.

use std::cmp::Ordering;

use super::error::{NativeError, NativeResult};
use crate::{StaticEnvironment, Value};

/// Extends a [`StaticEnvironment`] with `common` functions.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("all", None, all);
    env.add_native_func("any", None, any);
    env.add_native_func("between", Some(3), between);
    env.add_native_func("bool", Some(1), bool);
    env.add_native_func("contains", Some(2), contains);
    env.add_native_func("compare", Some(2), compare);
    env.add_native_func("empty", Some(1), empty);
    env.add_native_func("float", Some(1), float);
    env.add_native_func("if_then", Some(2), if_then);
    env.add_native_func("insert", Some(3), insert);
    env.add_native_func("int", Some(1), int);
    env.add_native_func("length", Some(1), length);
    env.add_native_func("max", None, max);
    env.add_native_func("min", None, min);
    env.add_native_func("reverse", Some(1), reverse);
    env.add_native_func("str", Some(1), str);
}

/// Return the first parameter if it's an [`Value::Array`] or return all
/// parameters as varadic function.
fn smart_vec(params: &[Value]) -> &[Value] {
    match params.first() {
        Some(Value::Array(v)) if (params.len() == 1) => v, // only one Array parameter
        _ => params,                                       // all varadic params
    }
}

/// Checks if all members of a [`Value::Array`] are [`Value::Boolean(true)`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
#[allow(clippy::missing_errors_doc)]
pub fn all(params: &[Value]) -> NativeResult {
    let values = smart_vec(params);
    let result = values.iter().all(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

/// Checks if any member of a [`Value::Array`] is [`Value::Boolean(true)`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
#[allow(clippy::missing_errors_doc)]
pub fn any(params: &[Value]) -> NativeResult {
    let values = smart_vec(params);
    let result = values.iter().any(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

/// Returns a [`Value::Boolean`] indicating if the first parameter falls within
/// the range of the second and third parameter.
///
/// # Remarks
///
/// The range includes the lower and upper bounds.
///
/// # Errors
///
/// Returns an error if there are not enough parameters.
pub fn between(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1), params.get(2)) {
        (Some(value), Some(lower), Some(upper)) => {
            let result = value >= lower && value <= upper;

            Ok(Value::Boolean(result))
        }
        _ => Err(NativeError::NotEnoughParameters(3)),
    }
}

/// Converts any [`Value`] to a [`Value::Boolean`].
///
/// # Remarks
///
/// Conversion depends on the supplied [`Value`] parameter:
/// * [`Value::Boolean`]: stays the same
/// * [`Value::Number`]: true = 1.0
/// * [`Value::String`]: true = "true" (case insensitive)
/// * [`Value::Array`]: true = !is_empty()
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied.
pub fn bool(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => match value {
            Value::String(v) => Ok(Value::Boolean(v.to_lowercase() == "true")), // "true" => true, other => false
            Value::Number(v) => Ok(Value::Boolean(
                v.partial_cmp(&1.0).is_some_and(|o| o == Ordering::Equal), // 1 => true, other (incl. NaN) => false
            )),
            Value::Array(v) => Ok(Value::Boolean(!v.is_empty())), // [] => false, other => true
            Value::Boolean(v) => Ok(Value::Boolean(*v)),          // Boolean => Boolean
        },
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Checks if the second parameter (needle) is contained inside the first (haystack).
/// Can be called with either:
/// * two [`Value::String`]
/// * a [`Value::Array`] as haystack and any [`Value`] as needle
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn contains(params: &[Value]) -> NativeResult {
    let found = match (params.get(0), params.get(1)) {
        (Some(haystack), Some(needle)) => match (haystack, needle) {
            (Value::String(needle), Value::String(haystack)) => needle.contains(haystack), // search in String
            (Value::Array(haystack), needle) => haystack.iter().any(|v| v == needle), // search in Array
            _ => return Err(NativeError::WrongParameterType),
        },
        _ => return Err(NativeError::NotEnoughParameters(2)),
    };

    Ok(Value::Boolean(found))
}

/// Compares two [`Value`] and returns the [`Ordering`] as [`Value::Number`].
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the [`Value`] are
/// not comparable.
pub fn compare(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1)) {
        (Some(left), Some(right)) => Ok(Value::Number(f64::from(
            left.partial_cmp(right)
                .ok_or(String::from("not comparable"))? as i8,
        ))),
        _ => Err(NativeError::NotEnoughParameters(2)),
    }
}

/// Checks if supplied [`Value`] is empty.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied.
pub fn empty(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => Ok(Value::Boolean(value.is_empty())),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts [`Value::Boolean`] or [`Value::String`] to a [`Value::Number`].
/// A [`Value::Number`] will retain it's value.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn float(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => match value {
            Value::Boolean(v) => Ok(Value::Number(f64::from(*v as i8))),
            Value::String(v) => {
                let float = v.parse::<f64>().map_err(|e| e.to_string())?;
                Ok(Value::Number(float))
            }
            Value::Number(v) => Ok(Value::Number(*v)),
            _ => Err(NativeError::WrongParameterType),
        },
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// If the first parameter is [`Value::Boolean(true)`] returns the second parameter, otherwise returns the thrid.
/// If the third parameter is not defined, return an empty [`Value`] of the same type as the second parameter.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
pub fn if_then(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1), params.get(2)) {
        (Some(Value::Boolean(condition)), Some(first), second) => {
            if *condition {
                Ok(first.clone())
            } else {
                Ok(second.cloned().unwrap_or_else(|| first.empty()))
            }
        }
        (Some(_), _, _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(2)),
    }
}

/// Inserts a [`Value::String`] into another [`Value::String`] at the specified
/// character index.
/// Or Inserts a [`Value`] into a [`Value::Array`] at the specified index.
///
/// # Errors
///
/// Returns an error if there are not enough parameters, the parameters are of
/// the wrong [`Value`] type or if the index is out of bounds.
pub fn insert(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1), params.get(2)) {
        (Some(Value::Array(values)), Some(element), Some(Value::Number(index))) => {
            let index = *index as usize;
            if index > values.len() {
                return Err(NativeError::IndexOutOfBounds(index));
            }

            let mut values = values.clone();
            values.insert(index, element.clone());

            Ok(Value::Array(values))
        }
        (Some(Value::String(target)), Some(Value::String(source)), Some(Value::Number(index))) => {
            let index = *index as usize;
            if index > target.chars().count() {
                return Err(NativeError::IndexOutOfBounds(index));
            }

            let before: String = target.chars().take(index).collect();
            let after: String = target.chars().skip(index).collect();

            Ok(Value::String(before + source + &after))
        }
        (Some(_), _, Some(_)) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(3)),
    }
}

/// Converts any [`Value`] to a [`Value::Number`] with integer precision.
/// See [`float`] for conversion information.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn int(params: &[Value]) -> NativeResult {
    match float(params)? {
        Value::Number(value) => Ok(Value::Number(value.trunc())),
        _ => Err(NativeError::WrongParameterType),
    }
}

/// Returns the length of the supplied [`Value::String`] or [`Value::Array`].
/// For other [`Value`] types return 0.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied.
pub fn length(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => Ok(Value::Number(value.len() as f64)),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Returns the maximum [`Value`] of a [`Value::Array`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
///
/// # Errors
///
/// Returns an error if the [`Value::Array`] can not be sorted.
pub fn max(params: &[Value]) -> NativeResult {
    smart_vec(params)
        .iter()
        .max_by(|a, b| {
            if a < b {
                Ordering::Less
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .cloned()
        .ok_or(NativeError::NotEnoughParameters(1))
}

/// Returns the minimum [`Value`] of a [`Value::Array`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
///
/// # Errors
///
/// Returns an error if the [`Value::Array`] can not be sorted.
pub fn min(params: &[Value]) -> NativeResult {
    smart_vec(params)
        .iter()
        .min_by(|a, b| {
            if a < b {
                Ordering::Less
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        })
        .cloned()
        .ok_or(NativeError::NotEnoughParameters(1))
}

/// Reverses the items of a [`Value::Array`] or the characters of a [`Value::String`].
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// is not reversible.
pub fn reverse(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::Array(values)) => Ok(Value::Array(values.iter().cloned().rev().collect())),
        Some(Value::String(value)) => Ok(Value::String(value.chars().rev().collect())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts any [`Value`] to a [`Value::String`].
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn str(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(value) => Ok(Value::String(value.to_string())),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::value::Value;

    #[test]
    fn std_all() {
        let values = vec![Value::Boolean(true), Value::Boolean(true)];
        assert_eq!(Value::Boolean(true), all(&values).unwrap());

        let values = vec![Value::Boolean(true), Value::Boolean(false)];
        assert_eq!(Value::Boolean(false), all(&values).unwrap());

        let values = vec![Value::Array(vec![
            Value::Boolean(true),
            Value::Boolean(true),
        ])];
        assert_eq!(Value::Boolean(true), all(&values).unwrap());

        let values = vec![Value::Array(vec![
            Value::Boolean(true),
            Value::Boolean(false),
        ])];
        assert_eq!(Value::Boolean(false), all(&values).unwrap());
    }

    #[test]
    fn std_any() {
        let values = vec![Value::Boolean(true), Value::Boolean(true)];
        assert_eq!(Value::Boolean(true), any(&values).unwrap());

        let values = vec![Value::Boolean(true), Value::Boolean(false)];
        assert_eq!(Value::Boolean(true), any(&values).unwrap());

        let values = vec![Value::Boolean(false), Value::Boolean(false)];
        assert_eq!(Value::Boolean(false), any(&values).unwrap());

        let values = vec![Value::Array(vec![
            Value::Boolean(true),
            Value::Boolean(true),
        ])];
        assert_eq!(Value::Boolean(true), any(&values).unwrap());

        let values = vec![Value::Array(vec![
            Value::Boolean(true),
            Value::Boolean(false),
        ])];
        assert_eq!(Value::Boolean(true), any(&values).unwrap());

        let values = vec![Value::Array(vec![
            Value::Boolean(false),
            Value::Boolean(false),
        ])];
        assert_eq!(Value::Boolean(false), any(&values).unwrap());
    }

    #[test]
    #[rustfmt::skip]
    fn std_between() {
        assert_eq!(
            Ok(Value::Boolean(true)),
            between(&vec![Value::Number(2.0), Value::Number(1.0), Value::Number(3.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(true)),
            between(&vec![Value::Number(20.0), Value::Number(20.0), Value::Number(30.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(true)),
            between(&vec![Value::Number(3.0), Value::Number(1.0), Value::Number(3.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(true)),
            between(&vec![Value::Number(-5.0), Value::Number(-6.0), Value::Number(-4.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            between(&vec![Value::Number(4.0), Value::Number(1.0), Value::Number(3.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(true)),
            between(&vec![Value::String(String::from("b")), Value::String(String::from("a")), Value::String(String::from("c"))])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            between(&vec![Value::String(String::from("a")), Value::String(String::from("b")), Value::String(String::from("c"))])
        );
    }

    #[test]
    fn std_bool() {
        assert_eq!(
            Value::Boolean(false),
            bool(&vec![Value::Number(0.0)]).unwrap()
        );

        assert_eq!(
            Value::Boolean(true),
            bool(&vec![Value::Number(1.0)]).unwrap()
        );

        assert_eq!(
            Value::Boolean(true),
            bool(&vec![Value::String(String::from("true"))]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            bool(&vec![Value::String(String::from("other"))]).unwrap()
        );

        assert_eq!(
            Value::Boolean(true),
            bool(&vec![Value::Boolean(true)]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            bool(&vec![Value::Array(vec![])]).unwrap()
        );

        assert!(bool(&vec![]).is_err());
    }

    #[test]
    fn std_contains_array() {
        let values = vec![
            Value::Number(30.0),
            Value::Number(10.0),
            Value::Number(20.0),
        ];

        assert_eq!(
            Ok(Value::Boolean(true)),
            contains(&vec![Value::Array(values.clone()), Value::Number(10.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            contains(&vec![Value::Array(values), Value::Number(11.0)])
        );

        assert!(contains(&vec![Value::Boolean(true), Value::Boolean(false)]).is_err());
        assert!(contains(&vec![]).is_err());
    }

    #[test]
    fn std_contains_string() {
        assert_eq!(
            Ok(Value::Boolean(true)),
            contains(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from("World"))
            ])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            contains(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from("WORLD"))
            ])
        );

        assert!(min(&vec![]).is_err());
    }

    #[test]
    fn std_compare() {
        assert_eq!(
            Ok(Value::Number(-1.0)),
            compare(&vec![Value::Number(10.0), Value::Number(20.0)])
        );

        assert_eq!(
            Ok(Value::Number(0.0)),
            compare(&vec![Value::Number(15.0), Value::Number(15.0)])
        );

        assert_eq!(
            Ok(Value::Number(1.0)),
            compare(&vec![Value::Number(20.0), Value::Number(10.0)])
        );
    }

    #[test]
    fn std_empty() {
        assert_eq!(
            Value::Boolean(true),
            empty(&vec![Value::String(String::from(""))]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            empty(&vec![Value::String(String::from("🙄"))]).unwrap()
        );

        assert_eq!(
            Value::Boolean(true),
            empty(&vec![Value::Array(vec![])]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            empty(&vec![Value::Array(vec![Value::Boolean(false)])]).unwrap()
        );

        assert!(empty(&vec![]).is_err());
    }

    #[test]
    fn std_float() {
        assert_eq!(
            Value::Number(12.2),
            float(&vec![Value::String(String::from("12.2"))]).unwrap()
        );

        assert_eq!(
            Value::Number(-12.2),
            float(&vec![Value::String(String::from("-12.2"))]).unwrap()
        );

        assert_eq!(
            Value::Number(0.123),
            float(&vec![Value::String(String::from(".123"))]).unwrap()
        );

        assert_eq!(Ok(Value::Number(1.0)), float(&vec![Value::Boolean(true)]));
        assert_eq!(Ok(Value::Number(0.0)), float(&vec![Value::Boolean(false)]));

        assert!(float(&vec![]).is_err());
    }

    #[test]
    fn std_if_then() {
        assert_eq!(
            Ok(Value::Number(1.0)),
            if_then(&vec![
                Value::Boolean(true),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );

        assert_eq!(
            Ok(Value::Number(2.0)),
            if_then(&vec![
                Value::Boolean(false),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );

        assert_eq!(
            Ok(Value::Number(1.0)),
            if_then(&vec![Value::Boolean(true), Value::Number(1.0)])
        );

        assert_eq!(
            Ok(Value::Number(0.0)),
            if_then(&vec![Value::Boolean(false), Value::Number(1.0)])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            if_then(&vec![Value::Boolean(false), Value::Boolean(true)])
        );

        assert_eq!(
            Ok(Value::String(String::new())),
            if_then(&vec![
                Value::Boolean(false),
                Value::String(String::from(String::from("Hello World")))
            ])
        );

        assert_eq!(
            Ok(Value::Array(vec![])),
            if_then(&vec![
                Value::Boolean(false),
                Value::Array(vec![Value::Boolean(true)]),
            ])
        );
    }

    #[test]
    fn std_insert() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::String(String::from("Hello")),
                Value::String(String::from("middle")),
                Value::String(String::from("world"))
            ])),
            insert(&vec![
                Value::Array(vec![
                    Value::String(String::from("Hello")),
                    Value::String(String::from("world"))
                ]),
                Value::String(String::from("middle")),
                Value::Number(1.0)
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("Hello middle world"))),
            insert(&vec![
                Value::String(String::from("Hello world")),
                Value::String(String::from("middle ")),
                Value::Number(6.0)
            ])
        );
    }

    #[test]
    fn std_int() {
        assert_eq!(
            Value::Number(12.0),
            int(&vec![Value::String(String::from("12.2"))]).unwrap()
        );

        assert_eq!(
            Value::Number(-12.0),
            int(&vec![Value::String(String::from("-12.2"))]).unwrap()
        );

        assert_eq!(
            Value::Number(0.0),
            int(&vec![Value::String(String::from(".123"))]).unwrap()
        );

        assert_eq!(Ok(Value::Number(1.0)), int(&vec![Value::Boolean(true)]));
        assert_eq!(Ok(Value::Number(0.0)), int(&vec![Value::Boolean(false)]));

        assert!(int(&vec![]).is_err());
    }

    #[test]
    fn std_length() {
        assert_eq!(Ok(Value::Number(0.0)), length(&vec![Value::Boolean(true)]));
        assert_eq!(Ok(Value::Number(0.0)), length(&vec![Value::Number(100.0)]));

        assert_eq!(
            Ok(Value::Number(5.0)),
            length(&vec![Value::String(String::from("Hello"))])
        );

        assert_eq!(
            Ok(Value::Number(2.0)),
            length(&vec![Value::Array(vec![
                Value::Boolean(true),
                Value::Boolean(false)
            ])])
        );

        assert!(length(&vec![]).is_err());
    }

    #[test]
    fn std_max() {
        let values = vec![Value::Number(10.0), Value::Number(20.0)];
        assert_eq!(Value::Number(20.0), max(&values).unwrap());

        let values = vec![
            Value::Number(30.0),
            Value::Number(10.0),
            Value::Number(20.0),
        ];
        assert_eq!(Value::Number(30.0), max(&values).unwrap());

        let values = vec![
            Value::Number(10.0),
            Value::Number(10.0),
            Value::Number(20.0),
        ];
        assert_eq!(Value::Number(20.0), max(&values).unwrap());

        assert!(max(&vec![]).is_err());
    }

    #[test]
    fn std_min() {
        let values = vec![Value::Number(10.0), Value::Number(20.0)];
        assert_eq!(Value::Number(10.0), min(&values).unwrap());

        let values = vec![
            Value::Number(30.0),
            Value::Number(10.0),
            Value::Number(20.0),
        ];
        assert_eq!(Value::Number(10.0), min(&values).unwrap());

        let values = vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(20.0),
        ];
        assert_eq!(Value::Number(10.0), min(&values).unwrap());

        assert!(min(&vec![]).is_err());
    }

    #[test]
    fn std_rev() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::Number(3.0),
                Value::Number(2.0),
                Value::Number(1.0)
            ])),
            reverse(&vec![Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])])
        );

        assert_eq!(
            Ok(Value::String(String::from("😎 dlroW olleH"))),
            reverse(&vec![Value::String(String::from("Hello World 😎"))])
        );
    }

    #[test]
    fn std_str() {
        assert_eq!(
            Ok(Value::String(String::from("123"))),
            str(&vec![Value::String(String::from("123"))])
        );

        assert_eq!(
            Ok(Value::String(String::from("123"))),
            str(&vec![Value::Number(123.0)])
        );

        assert_eq!(
            Ok(Value::String(String::from("true"))),
            str(&vec![Value::Boolean(true)])
        );

        assert!(str(&vec![]).is_err());
    }
}
