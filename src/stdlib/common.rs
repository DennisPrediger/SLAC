//! Common functions and constants for converting variables into different
//! [`Value`] types or check, extract and extend [`Value::Array`] variables.

use std::cmp::Ordering;

use super::{
    default_string,
    error::{NativeError, NativeResult},
    get_index, get_string_index, STRING_OFFSET,
};

use crate::{
    environment::{Arity, Function},
    Value,
};

/// Returns all common functions as a fixed size array.
#[rustfmt::skip]
pub fn functions() -> Vec<Function> {
    vec![
        Function::new(all, Arity::Variadic, "all(...): Boolean"),
        Function::new(any, Arity::Variadic, "any(...): Boolean"),
        Function::new(at, Arity::required(2), "at(values: [String|Array], index: Number): Any"),
        Function::new(between, Arity::required(3), "between(value: Any, lower: Any, upper: Any): Boolean"),
        Function::new(bool, Arity::required(1), "bool(value: Any): Boolean"),
        Function::new(contains, Arity::required(2), "contains(haystack: [String|Array], needle: [String|Any]): Boolean"),
        Function::new(compare, Arity::required(2), "compare(left: Any, right: Any): Number"),
        Function::new(copy, Arity::required(3), "copy(source: [String|Array], start: Number, count: Number): [String|Array]"),
        Function::new(empty, Arity::required(1), "empty(value: Any): Boolean"),
        Function::new(find, Arity::required(2), "find(haystack: [String|Array], needle: [String|Any]): Number"),
        Function::new(float, Arity::required(1), "float(value: Any): Number"),
        Function::new(if_then, Arity::optional(2, 1), "if_then(condition: Boolean, first: Any, second: Any): Any"),
        Function::new(insert, Arity::required(3), "insert(target: [String|Array], source: [String|Any], index: Number): Any"),
        Function::new(int, Arity::required(1), "int(value: Any): Number"),
        Function::new(length, Arity::required(1), "length(value: [String|Array]): Number"),
        Function::new(max, Arity::Variadic, "max(...): Any"),
        Function::new(min, Arity::Variadic, "min(...): Any"),
        Function::new(replace, Arity::optional(2, 1), "replace(value: [String|Array], from: [String|Any], to: [String|Any]): [String|Array]"),
        Function::new(replace, Arity::required(2), "remove(value: [String|Array], from: [String|Any]): [String|Array]"), // replace with only 2 parameters acts as remove
        Function::new(reverse, Arity::required(1), "reverse(value: [Array|String]): [Array|String]"),
        Function::new(str, Arity::required(1), "str(value: Any): String"),
    ]
}

/// Return the first parameter if it's an [`Value::Array`] or return all
/// parameters as varadic function.
fn smart_vec(params: &[Value]) -> &[Value] {
    match params {
        [Value::Array(v)] if (params.len() == 1) => v, // only one Array parameter
        _ => params,                                   // all varadic params
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

/// Returns the item at the specified index.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn at(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(values), Value::Number(index)] => {
            let index = get_string_index(index)?;

            match values.chars().nth(index) {
                Some(char) => Ok(Value::String(char.to_string())),
                None => Err(NativeError::IndexOutOfBounds(index)),
            }
        }
        [Value::Array(values), Value::Number(index)] => {
            let index = get_index(index)?;

            match values.get(index) {
                Some(value) => Ok(value.clone()),
                None => Err(NativeError::IndexOutOfBounds(index)),
            }
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
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
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn between(params: &[Value]) -> NativeResult {
    match params {
        [value, lower, upper] => Ok(Value::Boolean((value >= lower) && (value <= upper))),
        _ => Err(NativeError::WrongParameterCount(3)),
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
/// * [`Value::Array`]: true = `!is_empty()`
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn bool(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(v)] => Ok(Value::Boolean(v.to_lowercase() == "true")), // "true" => true, other => false
        [Value::Number(v)] => Ok(Value::Boolean(
            v.partial_cmp(&1.0).is_some_and(|o| o == Ordering::Equal), // 1 => true, other (incl. NaN) => false
        )),
        [Value::Array(v)] => Ok(Value::Boolean(!v.is_empty())), // [] => false, other => true
        [Value::Boolean(v)] => Ok(Value::Boolean(*v)),          // Boolean => Boolean
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Checks if the second parameter (needle) is contained inside the first (haystack).
/// Can be called with either:
/// * two [`Value::String`]
/// * a [`Value::Array`] as haystack and any [`Value`] as needle
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn contains(params: &[Value]) -> NativeResult {
    let found = match params {
        [Value::String(haystack), Value::String(needle)] => haystack.contains(needle), // search in String
        [Value::Array(haystack), needle] => haystack.iter().any(|v| v == needle), // search in Array
        [_, _] => return Err(NativeError::WrongParameterType),
        _ => return Err(NativeError::WrongParameterCount(2)),
    };

    Ok(Value::Boolean(found))
}

/// Compares two [`Value`] and returns the [`Ordering`] as [`Value::Number`].
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn compare(params: &[Value]) -> NativeResult {
    match params {
        [left, right] => Ok(Value::Number(f64::from(
            left.partial_cmp(right)
                .ok_or(NativeError::from("values not comparable"))? as i8,
        ))),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Copys a range of a [`Value::String`] or [`Value::Array`].
/// The first parameter sets the start index, the second the number of characters to copy.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn copy(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(source), Value::Number(start), Value::Number(count)] => Ok(Value::String(
            source
                .chars()
                .skip(get_string_index(start)?)
                .take(count.trunc() as usize)
                .collect(),
        )),
        [Value::Array(source), Value::Number(start), Value::Number(count)] => Ok(Value::Array(
            source
                .iter()
                .skip(get_index(start)?)
                .take(count.trunc() as usize)
                .cloned()
                .collect(),
        )),
        [_, _, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(3)),
    }
}

/// Checks if the supplied [`Value`] is empty.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn empty(params: &[Value]) -> NativeResult {
    match params {
        [value] => Ok(Value::Boolean(value.is_empty())),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Finds the index of a [`Value`] inside an [`Value::Array`] or the position of a substring inside
/// a [`Value::String`].
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn find(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(haystack), Value::String(needle)] => Ok(haystack
            .find(needle)
            .map_or(Value::Number(-1.0 + STRING_OFFSET), |index| {
                Value::Number((index as f64) + STRING_OFFSET)
            })),
        [Value::Array(haystack), needle] => Ok(haystack
            .iter()
            .position(|v| v == needle)
            .map_or(Value::Number(-1.0), |index| Value::Number(index as f64))),
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Converts [`Value::Boolean`] or [`Value::String`] to a [`Value::Number`].
/// A [`Value::Number`] will retain it's value.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn float(params: &[Value]) -> NativeResult {
    match params {
        [Value::Boolean(v)] => Ok(Value::Number(f64::from(i8::from(*v)))),
        [Value::String(v)] => {
            let float = v.parse::<f64>().map_err(|e| e.to_string())?;
            Ok(Value::Number(float))
        }
        [Value::Number(v)] => Ok(Value::Number(*v)),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// If the first parameter is [`Value::Boolean(true)`] returns the second parameter, otherwise returns the thrid.
/// If the third parameter is not defined, return an empty [`Value`] of the same type as the second parameter.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn if_then(params: &[Value]) -> NativeResult {
    match params {
        [Value::Boolean(condition), first, ..] => {
            if *condition {
                Ok(first.clone())
            } else {
                Ok(params.get(2).cloned().unwrap_or_else(|| first.empty()))
            }
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Inserts a [`Value::String`] into another [`Value::String`] at the specified
/// character index.
/// Or Inserts a [`Value`] into a [`Value::Array`] at the specified index.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
/// Will return [`NativeError::IndexOutOfBounds`] if the index parameter does not fit inside the supplied value length.
pub fn insert(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(target), Value::String(source), Value::Number(index)] => {
            let index = get_string_index(index)?;

            if index > target.chars().count() {
                return Err(NativeError::IndexOutOfBounds(index));
            }

            let before: String = target.chars().take(index).collect();
            let after: String = target.chars().skip(index).collect();

            Ok(Value::String(before + source + &after))
        }
        [Value::Array(values), element, Value::Number(index)] => {
            let index = get_index(index)?;
            if index > values.len() {
                return Err(NativeError::IndexOutOfBounds(index));
            }

            let mut values = values.clone();
            values.insert(index, element.clone());

            Ok(Value::Array(values))
        }
        [_, _, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(3)),
    }
}

/// Converts any [`Value`] to a [`Value::Number`] with integer precision.
/// See [`float`] for conversion information.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
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
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn length(params: &[Value]) -> NativeResult {
    match params {
        [value] => Ok(Value::Number(value.len() as f64)),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Returns the maximum [`Value`] of a [`Value::Array`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
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
        .ok_or(NativeError::WrongParameterCount(1))
}

/// Returns the minimum [`Value`] of a [`Value::Array`].
/// Can be called with a single [`Value::Array`] parameter or as varadic function.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
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
        .ok_or(NativeError::WrongParameterCount(1))
}

/// Reverses the items of a [`Value::Array`] or the characters of a [`Value::String`].
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn reverse(params: &[Value]) -> NativeResult {
    match params {
        [Value::Array(values)] => Ok(Value::Array(values.iter().cloned().rev().collect())),
        [Value::String(value)] => Ok(Value::String(value.chars().rev().collect())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Replaces all matches of a pattern with another string.
///
/// # Remarks
///
/// If a third parameter is not supplied the replacement will be an empty string.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn replace(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value), Value::String(from), ..] => {
            let to = default_string(params, 2, "")?;
            Ok(Value::String(value.replace(from, to)))
        }
        [Value::Array(values), from, ..] => {
            let to = params.get(2).cloned();

            Ok(Value::Array(
                values
                    .iter()
                    .filter_map(|value| {
                        if value == from {
                            to.clone()
                        } else {
                            Some(value.clone())
                        }
                    })
                    .collect(),
            ))
        }
        [_, _, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(3)),
    }
}

/// Converts any [`Value`] to a [`Value::String`].
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
pub fn str(params: &[Value]) -> NativeResult {
    match params {
        [value] => Ok(Value::String(value.to_string())),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            Ok(Value::String(String::from("12A345"))),
            insert(&vec![
                Value::String(String::from("12345")),
                Value::String(String::from("A")),
                Value::Number(2.0 + STRING_OFFSET)
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("Hello middle world"))),
            insert(&vec![
                Value::String(String::from("Hello world")),
                Value::String(String::from("middle ")),
                Value::Number(6.0 + STRING_OFFSET)
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

    #[test]
    fn std_copy() {
        assert_eq!(
            Ok(Value::String(String::from("Worl"))),
            copy(&vec![
                Value::String(String::from("Hello World")),
                Value::Number(6.0 + STRING_OFFSET),
                Value::Number(4.0)
            ])
        );

        assert_eq!(
            Ok(Value::Array(vec![Value::Number(2.0), Value::Number(3.0),])),
            copy(&vec![
                Value::Array(vec![
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                    Value::Number(4.0)
                ]),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );
    }

    #[test]
    fn std_at() {
        assert_eq!(
            Ok(Value::String(String::from("b"))),
            at(&vec![
                Value::String(String::from("abcde")),
                Value::Number(1.0 + STRING_OFFSET)
            ])
        );

        assert_eq!(
            Ok(Value::Number(2.0)),
            at(&vec![
                Value::Array(vec![
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]),
                Value::Number(1.0)
            ])
        );
    }

    #[test]
    fn std_find() {
        assert_eq!(
            Ok(Value::Number(3.0 + STRING_OFFSET)),
            find(&vec![
                Value::String(String::from("abcde")),
                Value::String(String::from("de"))
            ])
        );

        assert_eq!(
            Ok(Value::Number(-1.0 + STRING_OFFSET)),
            find(&vec![
                Value::String(String::from("abcde")),
                Value::String(String::from("f"))
            ])
        );

        assert_eq!(
            Ok(Value::Number(1.0)),
            find(&vec![
                Value::Array(vec![
                    Value::Boolean(true),
                    Value::Boolean(false),
                    Value::Boolean(true)
                ]),
                Value::Boolean(false)
            ])
        );

        assert_eq!(
            Ok(Value::Number(-1.0)),
            find(&vec![
                Value::Array(vec![
                    Value::Boolean(true),
                    Value::Boolean(false),
                    Value::Boolean(true)
                ]),
                Value::String(String::from("abc"))
            ])
        );
    }

    #[test]
    fn std_replace_string() {
        assert_eq!(
            Ok(Value::String(String::from("Hello Moon"))),
            replace(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from("World")),
                Value::String(String::from("Moon"))
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("Heiio Worid"))),
            replace(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from("l")),
                Value::String(String::from("i"))
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("Hello"))),
            replace(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from(" World")),
                Value::String(String::from(""))
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("Hello"))),
            replace(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from(" World"))
            ])
        );
    }

    #[test]
    fn std_replace_array() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::Number(1.0),
                Value::Number(1.0),
                Value::Number(3.0)
            ])),
            replace(&vec![
                Value::Array(vec![
                    Value::Number(1.0),
                    Value::Number(1.0),
                    Value::Number(3.0)
                ]),
                Value::Number(2.0),
                Value::Number(1.0)
            ])
        );
    }
}
