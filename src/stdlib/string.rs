//! Functions to manipulate [`Value::String`] variables.

use crate::{StaticEnvironment, Value};

use super::error::{NativeError, NativeResult};

/// Extends a [`StaticEnvironment`] with functions to manipulate [`Value::String`] variables.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_function("chr", chr, Some(1), 0);
    env.add_function("ord", ord, Some(1), 0);
    env.add_function("lowercase", lowercase, Some(1), 0);
    env.add_function("uppercase", uppercase, Some(1), 0);
    env.add_function("same_text", same_text, Some(2), 0);
    env.add_function("split", split, Some(2), 0);
    env.add_function("split_csv", split_csv, Some(2), 1);
    env.add_function("trim", trim, Some(1), 0);
    env.add_function("trim_left", trim_left, Some(1), 0);
    env.add_function("trim_right", trim_right, Some(1), 0);
}

/// Converts a [`Value::Number`] into a [`Value::String`] containg a single
/// ASCII character.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
/// Will return [`NativeError::CustomError`] if the supplied number is outside of ASCII character range.
pub fn chr(params: &[Value]) -> NativeResult {
    match params {
        [Value::Number(value)] if (0.0..127.0).contains(value) => Ok(Value::String(
            char::from_u32(*value as u32).unwrap_or('\0').to_string(),
        )),
        [Value::Number(_)] => Err(NativeError::from("number is out of ASCII range")),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Converts a single charachter [`Value::String`] into a [`Value::Number`]
/// containing it's ASCII number value.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
/// Will return [`NativeError::CustomError`] if the supplied number is outside of ASCII character range.
pub fn ord(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] if value.chars().count() == 1 => {
            if value.is_ascii() {
                Ok(Value::Number(f64::from(
                    value.chars().next().unwrap_or('\0') as u8,
                )))
            } else {
                Err(NativeError::from("character is out of ASCII range"))
            }
        }
        [Value::String(_)] => Err(NativeError::from("string is too long")),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Converts a [`Value::String`] to lowercase.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn lowercase(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => Ok(Value::String(value.to_lowercase())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Converts a [`Value::String`] to uppercase.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn uppercase(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => Ok(Value::String(value.to_uppercase())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Compares two [`Value::String`] by text content.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn same_text(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(left), Value::String(right)] => {
            Ok(Value::Boolean(left.to_lowercase() == right.to_lowercase()))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(2)),
    }
}

/// Splits a [`Value::String`] into a [`Value::Array`] according to a seperator string.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn split(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(line), Value::String(seperator)] => {
            let values = line
                .split(seperator)
                .map(String::from)
                .map(Value::String)
                .collect();

            Ok(Value::Array(values))
        }
        [_, _] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

fn char_from_value(value: &Value) -> Option<char> {
    match value {
        Value::String(string) if string.len() == 1 => string.chars().next(),
        _ => None,
    }
}

fn parse_csv(line: &str, separator: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        if c == separator && !in_quotes {
            result.push(field.clone());
            field.clear();
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else {
            field.push(c);
        }
    }

    result.push(field);
    result
}

/// Splits a csv [`Value::String`] into a [`Value::Array`] according to a seperator
/// character (Default: ';').
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn split_csv(params: &[Value]) -> NativeResult {
    let separator = params.get(1).and_then(char_from_value).unwrap_or(';');

    match params {
        [Value::String(line), ..] => {
            let values = parse_csv(line, separator)
                .into_iter()
                .map(Value::String)
                .collect();
            Ok(Value::Array(values))
        }
        [_, ..] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Trims the whitespace of a [`Value::String`] on both sides.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn trim(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => Ok(Value::String(value.trim().to_string())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Trims the whitespace of a [`Value::String`] on the left side of the String.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn trim_left(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => Ok(Value::String(value.trim_start().to_string())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

/// Trims the whitespace of a [`Value::String`] on the right side of the String.
///
/// # Errors
///
/// Will return [`NativeError::WrongParameterCount`] if there is a mismatch in the supplied parameters.
/// Will return [`NativeError::WrongParameterType`] if the the supplied parameters have the wrong type.
pub fn trim_right(params: &[Value]) -> NativeResult {
    match params {
        [Value::String(value)] => Ok(Value::String(value.trim_end().to_string())),
        [_] => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::WrongParameterCount(1)),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;

    #[test]
    fn string_ord() {
        assert_eq!(
            Ok(Value::Number(97.0)),
            ord(&vec![Value::String(String::from("a"))])
        );

        assert_eq!(
            Ok(Value::Number(13.0)),
            ord(&vec![Value::String(String::from("\r"))])
        );
        assert_eq!(
            Ok(Value::Number(10.0)),
            ord(&vec![Value::String(String::from("\n"))])
        );

        assert!(ord(&vec![Value::String(String::from("Hello World"))]).is_err());
        assert!(ord(&vec![Value::String(String::from("🙄"))]).is_err());
    }

    #[test]
    fn string_chr() {
        assert_eq!(
            Ok(Value::String(String::from("a"))),
            chr(&vec![Value::Number(97.0)])
        );

        assert_eq!(
            Ok(Value::String(String::from("\0"))),
            chr(&vec![Value::Number(0.0)])
        );

        assert!(chr(&vec![Value::Number(256.0)]).is_err());
    }

    #[test]
    fn string_lowercase() {
        assert_eq!(
            Ok(Value::String(String::from("hello world"))),
            lowercase(&vec![Value::String(String::from("Hello World"))])
        );

        assert!(lowercase(&vec![]).is_err());
        assert!(lowercase(&vec![Value::Boolean(true)]).is_err());
    }

    #[test]
    fn string_uppercase() {
        assert_eq!(
            Ok(Value::String(String::from("HELLO WORLD"))),
            uppercase(&vec![Value::String(String::from("Hello World"))])
        );

        assert!(uppercase(&vec![]).is_err());
        assert!(uppercase(&vec![Value::Boolean(true)]).is_err());
    }

    #[test]
    fn string_split_csv() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::String(String::from("Hello; World")),
                Value::String(String::from("1234")),
                Value::String(String::from("")),
                Value::String(String::from("End"))
            ])),
            split_csv(&vec![Value::String(String::from(
                "\"Hello; World\";1234;;End"
            ))])
        );

        assert_eq!(
            Ok(Value::Array(vec![Value::String(String::new())])),
            split_csv(&vec![Value::String(String::from(""))])
        );
    }

    #[test]
    fn string_split() {
        assert_eq!(
            Ok(Value::Array(vec![
                Value::String(String::from("\"Hello")),
                Value::String(String::from(" World\"")),
                Value::String(String::from("1234")),
                Value::String(String::from("")),
                Value::String(String::from("End"))
            ])),
            split(&vec![
                Value::String(String::from("\"Hello; World\";1234;;End")),
                Value::String(String::from(";"))
            ])
        );

        assert_eq!(
            Ok(Value::Array(vec![Value::String(String::new())])),
            split(&vec![
                Value::String(String::from("")),
                Value::String(String::from(";"))
            ])
        );
    }

    #[test]
    fn string_same_text() {
        assert_eq!(
            Ok(Value::Boolean(true)),
            same_text(&vec![
                Value::String(String::from("hello world")),
                Value::String(String::from("Hello World"))
            ])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            same_text(&vec![
                Value::String(String::from("hallo world")),
                Value::String(String::from("hello world"))
            ])
        );
    }

    #[test]
    fn string_trim() {
        assert_eq!(
            Ok(Value::String(String::from("Hello World"))),
            trim(&vec![Value::String(String::from("  Hello World       "))])
        );

        assert!(trim(&vec![]).is_err());
        assert!(trim(&vec![Value::Boolean(true)]).is_err());

        assert_eq!(
            Ok(Value::String(String::from("Hello World       "))),
            trim_left(&vec![Value::String(String::from("  Hello World       "))])
        );

        assert_eq!(
            Ok(Value::String(String::from("  Hello World"))),
            trim_right(&vec![Value::String(String::from("  Hello World       "))])
        );
    }
}
