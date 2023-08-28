use crate::{StaticEnvironment, Value};

pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("chr", Some(1), chr);
    env.add_native_func("ord", Some(1), ord);
    env.add_native_func("lowercase", Some(1), lowercase);
    env.add_native_func("uppercase", Some(1), uppercase);
    env.add_native_func("same_text", Some(2), same_text);
    env.add_native_func("trim", Some(1), trim);
    env.add_native_func("trim_left", Some(1), trim_left);
    env.add_native_func("trim_right", Some(1), trim_right);
}

pub fn chr(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::Number(value)) if (0.0..127.0).contains(value) => Ok(Value::String(
            char::from_u32(*value as u32).unwrap_or('\0').to_string(),
        )),
        Some(Value::Number(_)) => Err(String::from("number is out of ascii range")),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

pub fn ord(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) if value.chars().count() == 1 => {
            if value.is_ascii() {
                Ok(Value::Number(f64::from(
                    value.chars().next().unwrap_or('\0') as u8,
                )))
            } else {
                Err(String::from("string out of ascii range"))
            }
        }
        Some(Value::String(_)) => Err(String::from("string is too long")),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Converts a [`Value::String`] to lowercase.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn lowercase(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.to_lowercase())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Converts a [`Value::String`] to uppercase.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn uppercase(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.to_uppercase())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Compares a String
/// # Errors
/// Will return an error if not at least two parameters are supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn same_text(params: &[Value]) -> Result<Value, String> {
    match (params.get(0), params.get(1)) {
        (Some(Value::String(left)), Some(Value::String(right))) => {
            Ok(Value::Boolean(left.to_lowercase() == right.to_lowercase()))
        }
        _ => Err(String::from("no param supplied")),
    }
}

/// Trims the whitespace of a [`Value::String`] on both sides.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim().to_string())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Trims the whitespace of a [`Value::String`] on the start of the String.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_left(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim_start().to_string())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Trims the whitespace of a [`Value::String`] on the end of the String.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_right(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::String(value)) => Ok(Value::String(value.trim_end().to_string())),
        Some(_) => Err(String::from("wrong parameter type")),
        None => Err(String::from("no parameter supplied")),
    }
}

#[cfg(test)]
mod test {
    use super::{chr, lowercase, ord, same_text, trim, trim_left, trim_right, uppercase};
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
