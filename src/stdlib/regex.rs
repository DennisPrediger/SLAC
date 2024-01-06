//! Optional module to perform matching and replacement of regular expressions on [`Value::String`].
//!
//! # Regex
//!
//! This modules uses the [`regex_lite`] crate and can be included using the `regex` feature.

use regex_lite::{Captures, Regex};

use crate::{
    environment::{Arity, Function},
    Value,
};

use super::{default_number, default_string, NativeError, NativeResult};

/// Returns all regex functions as a fixed size array.
#[rustfmt::skip]
pub fn functions() -> Vec<Function> {
    vec![
        Function::new(is_match, Arity::required(2), "re_is_match(haystack: String, pattern: String): Boolean"),
        Function::new(find, Arity::required(2), "re_find(haystack: String, pattern: String): Array<String>"),
        Function::new(capture, Arity::required(2), "re_capture(haystack: String, pattern: String): Array<String>"),
        Function::new(replace, Arity::optional(2, 2), "re_replace(haystack: String, pattern: String, replacement: String = '', limit = 0): String"),
    ]
}

/// Checks if a regex pattern matches a [`Value::String`].
///
/// * Declaration: `re_is_match(haystack: String, pattern: String): Boolean`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the regex produces an error.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn is_match(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(haystack), Value::String(pattern)] => {
            let re = Regex::new(pattern).map_err(|e| NativeError::from(e.to_string()))?;

            Ok(Value::Boolean(re.is_match(haystack)))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Finds non overlapping matches for a given regex inside a [`Value::String`] and
/// returns a [`Value::Array`] containing all matches.
///
/// * Declaration: `re_find(haystack: String, pattern: String): Array<String>`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the regex produces an error.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn find(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(haystack), Value::String(pattern)] => {
            let re = Regex::new(pattern).map_err(|e| NativeError::from(e.to_string()))?;

            let groups: Vec<Value> = re
                .find_iter(haystack)
                .map(|m| Value::String(m.as_str().to_string()))
                .collect();

            Ok(Value::Array(groups))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Extract a [`Value::Array`] from a [`Captures`] struct while preserving empty captures
/// as empty strings.
fn get_capture_groups(captures: Captures) -> Vec<Value> {
    captures
        .iter()
        .map(|c| c.map_or("", |m| m.as_str()))
        .map(|m| Value::String(m.to_string()))
        .collect()
}

/// Returns the matches of all regex capture groups inside a [`Value::Array`].
/// The first element is the full match; Index 1 to N are the capture groups.
///
/// * Declaration: `re_capture(haystack: String, pattern: String): Array<String>`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the regex produces an error.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn capture(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(haystack), Value::String(pattern)] => {
            let re = Regex::new(pattern).map_err(|e| NativeError::from(e.to_string()))?;

            let groups: Vec<Value> = re.captures(haystack).map_or_else(
                || vec![Value::String(String::new()); re.captures_len()],
                get_capture_groups,
            );

            Ok(Value::Array(groups))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Replaces all matches of a regex pattern inside a [`Value::String`] with a replacement [`Value::String`].
///
/// * Declaration: `re_replace(haystack: String, pattern: String, replacement: String = '', limit = 0): String`
///
/// # Errors
///
/// Will return [`NativeError::CustomError`] if the regex produces an error.
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn replace(params: &[Value]) -> NativeResult {
    let replacement = default_string(params, 2, "")?;
    let limit = default_number(params, 3, 0.0)? as usize;

    match params {
        [Value::String(haystack), Value::String(needle), ..] => {
            let re = Regex::new(needle).map_err(|e| NativeError::from(e.to_string()))?;

            Ok(Value::String(
                re.replacen(haystack, limit, replacement).to_string(),
            ))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use crate::Value;

    #[test]
    fn re_is_match() {
        assert_eq!(
            Ok(Value::Boolean(true)),
            is_match(&vec![
                Value::String(String::from("Hello World")),
                Value::String(String::from(".*World"))
            ])
        );
        assert_eq!(
            Ok(Value::Boolean(true)),
            is_match(&vec![
                Value::String(String::from(
                    "I categorically deny having triskaidekaphobia."
                )),
                Value::String(String::from(r"\b\w{13}\b"))
            ])
        );
    }

    #[test]
    fn re_find() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::String(String::from("100")),
                Value::String(String::from("200")),
                Value::String(String::from("300"))
            ])),
            find(&vec![
                Value::String(String::from("10 20 30 100 200 300 1000 2000 3000")),
                Value::String(String::from(r"\b\d{3}\b"))
            ])
        );
    }

    #[test]
    fn re_capture() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::String(String::from("2023-09-30")),
                Value::String(String::from("2023")),
                Value::String(String::from("09")),
                Value::String(String::from("30"))
            ])),
            capture(&vec![
                Value::String(String::from("2023-09-30")),
                Value::String(String::from(r"(\d{4})-(\d{2})-(\d{2})"))
            ])
        );
    }

    #[test]
    fn re_replace() {
        assert_eq!(
            Ok(Value::String(String::from("9999-09-30"))),
            replace(&vec![
                Value::String(String::from("2023-09-30")),
                Value::String(String::from(r"\d{4}")),
                Value::String(String::from("9999"))
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("2023-9999-9999"))),
            replace(&vec![
                Value::String(String::from("2023-09-30")),
                Value::String(String::from(r"\b\d{2}\b")),
                Value::String(String::from("9999")),
            ])
        );

        assert_eq!(
            Ok(Value::String(String::from("2023-9999-30"))),
            replace(&vec![
                Value::String(String::from("2023-09-30")),
                Value::String(String::from(r"\b\d{2}\b")),
                Value::String(String::from("9999")),
                Value::Number(1.0)
            ])
        );
    }
}
