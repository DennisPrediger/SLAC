use crate::{StaticEnvironment, Value};

pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("lowercase", Some(1), lowercase);
    env.add_native_func("uppercase", Some(1), uppercase);
    env.add_native_func("same_text", Some(2), same_text);
    env.add_native_func("trim", Some(1), trim);
    env.add_native_func("trim_left", Some(1), trim_left);
    env.add_native_func("trim_right", Some(1), trim_right);
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
    use super::{lowercase, same_text, trim, trim_left, trim_right, uppercase};
    use crate::Value;

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
