//! Functions to manipulate [`Value::String`] variables.

use crate::{StaticEnvironment, Value};

use super::error::{NativeError, NativeResult};

/// Extends a [`StaticEnvironment`] with functions to manipulate [`Value::String`] variables.
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("chr", Some(1), chr);
    env.add_native_func("ord", Some(1), ord);
    env.add_native_func("lowercase", Some(1), lowercase);
    env.add_native_func("uppercase", Some(1), uppercase);
    env.add_native_func("replace", Some(3), replace);
    env.add_native_func("same_text", Some(2), same_text);
    env.add_native_func("trim", Some(1), trim);
    env.add_native_func("trim_left", Some(1), trim_left);
    env.add_native_func("trim_right", Some(1), trim_right);
}

/// Converts a [`Value::Number`] into a [`Value::String`] containg a single
/// ASCII character.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
/// Returns an error if the supplied number is outside of ASCII character range.
pub fn chr(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::Number(value)) if (0.0..127.0).contains(value) => Ok(Value::String(
            char::from_u32(*value as u32).unwrap_or('\0').to_string(),
        )),
        Some(Value::Number(_)) => Err(NativeError::from("number is out of ASCII range")),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts a single charachter [`Value::String`] into a [`Value::Number`]
/// containing it's ASCII number value.
///
/// # Errors
///
/// Returns an error if there are not enough parameters or the parameters are of
/// the wrong [`Value`] type.
/// Returns an error if the supplied [`Value::String`] is longer than one character
/// or not an ASCII charachter.
pub fn ord(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) if value.chars().count() == 1 => {
            if value.is_ascii() {
                Ok(Value::Number(f64::from(
                    value.chars().next().unwrap_or('\0') as u8,
                )))
            } else {
                Err(NativeError::from("character is out of ASCII range"))
            }
        }
        Some(Value::String(_)) => Err(NativeError::from("string is too long")),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts a [`Value::String`] to lowercase.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn lowercase(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.to_lowercase())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Converts a [`Value::String`] to uppercase.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn uppercase(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.to_uppercase())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
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
/// Will return an error if not at least three parameters are supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn replace(params: &[Value]) -> NativeResult {
    match (
        params.get(0),
        params.get(1),
        params.get(2).unwrap_or(&Value::String(String::new())),
    ) {
        (Some(Value::String(value)), Some(Value::String(from)), Value::String(to)) => {
            Ok(Value::String(value.replace(from, to)))
        }
        (Some(_), Some(_), _) => Err(NativeError::WrongParameterType),
        _ => Err(NativeError::NotEnoughParameters(3)),
    }
}

/// Compares two [`Value::String`] by text content.
///
/// # Errors
///
/// Will return an error if not at least two parameters are supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn same_text(params: &[Value]) -> NativeResult {
    match (params.get(0), params.get(1)) {
        (Some(Value::String(left)), Some(Value::String(right))) => {
            Ok(Value::Boolean(left.to_lowercase() == right.to_lowercase()))
        }
        _ => Err(NativeError::NotEnoughParameters(2)),
    }
}

/// Trims the whitespace of a [`Value::String`] on both sides.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim().to_string())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Trims the whitespace of a [`Value::String`] on the left side of the String.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_left(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim_start().to_string())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
    }
}

/// Trims the whitespace of a [`Value::String`] on the right side of the String.
///
/// # Errors
///
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_right(params: &[Value]) -> NativeResult {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim_end().to_string())),
        Some(_) => Err(NativeError::WrongParameterType),
        None => Err(NativeError::NotEnoughParameters(1)),
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
    fn string_replace() {
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
