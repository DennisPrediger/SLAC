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
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.to_lowercase()))
    } else {
        Err("no param supplied".to_string())
    }
}

/// Converts a [`Value::String`] to uppercase.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn uppercase(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.to_uppercase()))
    } else {
        Err("no parameter supplied".to_string())
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
        _ => Err("no param supplied".to_string()),
    }
}

/// Trims the whitespace of a [`Value::String`] on both sides.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.trim().to_string()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

/// Trims the whitespace of a [`Value::String`] on the start of the String.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_left(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.trim_start().to_string()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

/// Trims the whitespace of a [`Value::String`] on the end of the String.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// [`Value`] is not a [`Value::String`]
pub fn trim_right(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.trim_end().to_string()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

#[cfg(test)]
mod test {
    use super::{lowercase, same_text, trim, trim_left, trim_right, uppercase};
    use crate::Value;

    #[test]
    fn string_lowercase() {
        assert_eq!(
            Ok(Value::String("hello world".to_string())),
            lowercase(&vec![Value::String("Hello World".to_string())])
        );

        assert!(lowercase(&vec![]).is_err());
        assert!(lowercase(&vec![Value::Boolean(true)]).is_err());
    }

    #[test]
    fn string_uppercase() {
        assert_eq!(
            Ok(Value::String("HELLO WORLD".to_string())),
            uppercase(&vec![Value::String("Hello World".to_string())])
        );

        assert!(uppercase(&vec![]).is_err());
        assert!(uppercase(&vec![Value::Boolean(true)]).is_err());
    }

    #[test]
    fn string_same_text() {
        assert_eq!(
            Ok(Value::Boolean(true)),
            same_text(&vec![
                Value::String("hello world".to_string()),
                Value::String("Hello World".to_string())
            ])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            same_text(&vec![
                Value::String("hallo world".to_string()),
                Value::String("hello world".to_string())
            ])
        );
    }

    #[test]
    fn string_trim() {
        assert_eq!(
            Ok(Value::String("Hello World".to_string())),
            trim(&vec![Value::String("  Hello World       ".to_string())])
        );

        assert!(trim(&vec![]).is_err());
        assert!(trim(&vec![Value::Boolean(true)]).is_err());

        assert_eq!(
            Ok(Value::String("Hello World       ".to_string())),
            trim_left(&vec![Value::String("  Hello World       ".to_string())])
        );

        assert_eq!(
            Ok(Value::String("  Hello World".to_string())),
            trim_right(&vec![Value::String("  Hello World       ".to_string())])
        );
    }
}
