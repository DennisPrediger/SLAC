//! Various common functions and constants to insert into a [`StaticEnvironment`].

use std::cmp::Ordering;

use crate::{StaticEnvironment, Value};

/// Insert all functions and constants into an [`StaticEnvironment`].
pub fn extend_environment(env: &mut StaticEnvironment) {
    env.add_native_func("all", None, all);
    env.add_native_func("any", None, any);
    env.add_native_func("bool", Some(1), bool);
    env.add_native_func("contains", Some(2), contains);
    env.add_native_func("compare", Some(2), compare);
    env.add_native_func("empty", Some(1), empty);
    env.add_native_func("float", Some(1), float);
    env.add_native_func("high", Some(1), high);
    env.add_native_func("low", Some(1), low);
    env.add_native_func("insert", Some(3), insert);
    env.add_native_func("int", Some(1), int);
    env.add_native_func("length", Some(1), length);
    env.add_native_func("max", None, max);
    env.add_native_func("min", None, min);
    env.add_native_func("str", Some(1), str);
}

fn smart_vec(params: &[Value]) -> &[Value] {
    match params.first() {
        Some(Value::Array(v)) if (params.len() == 1) => v, // only one Array parameter
        _ => params,                                       // all varadic params
    }
}

/// Checks if all members of an array are true.
/// Can be called either with a single [`Value::Array`] or variable list of Parameters.
///
/// # Errors
/// Always returns [`Ok(Value::Boolean)`].
pub fn all(params: &[Value]) -> Result<Value, String> {
    let values = smart_vec(params);
    let result = values.iter().all(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

/// Checks if any members of an array is true.
/// Can be called either with a single [`Value::Array`] or variable list of Parameters.
///
/// # Errors
/// Always returns [`Ok(Value::Boolean)`].
pub fn any(params: &[Value]) -> Result<Value, String> {
    let values = smart_vec(params);
    let result = values.iter().any(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

/// Converts any [`Value`] to a [`Value::Boolean`].
///
/// # Errors
/// Will return an error if not at least one parameter is supplied.
pub fn bool(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => match value {
            Value::String(v) => Ok(Value::Boolean(v.to_lowercase() == "true")), // "true" => true, other => false
            Value::Number(v) => Ok(Value::Boolean(
                v.partial_cmp(&1.0).is_some_and(|o| o == Ordering::Equal), // 1 => true, other (incl. NaN) => false
            )),
            Value::Array(v) => Ok(Value::Boolean(!v.is_empty())), // [] => false, other => true
            Value::Boolean(v) => Ok(Value::Boolean(*v)),          // Boolean => Boolean
        },
        None => Err(String::from("not enough parameters")),
    }
}

/// Checks if the second parameter (needle) is contained inside the first (haystack).
/// Can be called with either:
/// * two [`Value::String`]
/// * a [`Value::Array`] as haystack and any [`Value`] as needle
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or the supplied
/// parameters are of the wrong type.
pub fn contains(params: &[Value]) -> Result<Value, String> {
    let found = match (params.get(0), params.get(1)) {
        (Some(haystack), Some(needle)) => match (haystack, needle) {
            (Value::String(needle), Value::String(haystack)) => needle.contains(haystack), // search in String
            (Value::Array(haystack), needle) => haystack.iter().any(|v| v == needle), // search in Array
            _ => return Err(String::from("param types invalid")),
        },
        _ => return Err(String::from("not enough parameters")),
    };

    Ok(Value::Boolean(found))
}

pub fn compare(params: &[Value]) -> Result<Value, String> {
    match (params.get(0), params.get(1)) {
        (Some(left), Some(right)) => Ok(Value::Number(f64::from(
            left.partial_cmp(right)
                .ok_or(String::from("not comparable"))? as i8,
        ))),
        _ => Err(String::from("not enough parameters")),
    }
}

/// Checks if supplied [`Value`] is empty.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied.
pub fn empty(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => Ok(Value::Boolean(value.is_empty())),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Converts any [`Value`] to a [`Value::Number`] with floating point precision.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn float(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => match value {
            Value::Boolean(v) => Ok(Value::Number(f64::from(*v as i8))),
            Value::String(v) => {
                let float = v.parse::<f64>().map_err(|e| e.to_string())?;
                Ok(Value::Number(float))
            }
            Value::Number(v) => Ok(Value::Number(*v)),
            _ => Err(String::from("value can not be converted to float")),
        },
        None => Err(String::from("not enough parameters")),
    }
}

pub fn high(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::Array(values)) => values.last().cloned().ok_or(String::from("empty array")),
        Some(Value::String(value)) => Ok(Value::String(
            value.chars().last().unwrap_or_default().to_string(),
        )),
        Some(_) => Err(String::from("wrong parameter type")),
        _ => Err(String::from("not enough Parameters")),
    }
}

pub fn low(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::Array(values)) => values.first().cloned().ok_or(String::from("empty array")),
        Some(Value::String(value)) => Ok(Value::String(
            value.chars().next().unwrap_or_default().to_string(),
        )),
        Some(_) => Err(String::from("wrong parameter type")),
        _ => Err(String::from("not enough Parameters")),
    }
}

pub fn insert(params: &[Value]) -> Result<Value, String> {
    match (params.get(0), params.get(1), params.get(2)) {
        (Some(Value::Array(values)), Some(element), Some(Value::Number(index))) => {
            let index = *index as usize;
            if index > values.len() {
                return Err(String::from("index out of bounds"));
            }

            let mut values = values.clone();
            values.insert(index, element.clone());

            Ok(Value::Array(values))
        }
        (Some(Value::String(target)), Some(Value::String(source)), Some(Value::Number(index))) => {
            let index = *index as usize;
            if index > target.chars().count() {
                return Err(String::from("index out of bounds"));
            }

            let before: String = target.chars().take(index).collect();
            let after: String = target.chars().skip(index).collect();

            Ok(Value::String(before + source + &after))
        }
        (Some(_), _, Some(_)) => Err(String::from("wrong parameter type")),
        _ => Err(String::from("not enough Parameters")),
    }
}

/// Converts any [`Value`] to a [`Value::Number`] with integer precision.
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn int(params: &[Value]) -> Result<Value, String> {
    if let Value::Number(value) = float(params)? {
        Ok(Value::Number(value.trunc()))
    } else {
        Err(String::from("undefined input value"))
    }
}

/// Returns the length of the supplied Value
/// * [`Value::String`]: length of the String
/// * [`Value::Array`]: length of the Array
/// * otherwise 0
///
/// # Errors
/// Will return an error if not at least one parameter is supplied.
pub fn length(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => Ok(Value::Number(value.len() as f64)),
        None => Err(String::from("no parameter supplied")),
    }
}

/// Returns the maximum [`Value`] of a [`Value::Array`].
/// Can be called either with a single [`Value::Array`] or variable list of Parameters.
///
/// # Errors
/// Returns an error if the [`Value::Array`] can not be sorted.
pub fn max(params: &[Value]) -> Result<Value, String> {
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
        .ok_or(String::from("function 'max' failed"))
}

/// Returns the minimum [`Value`] of a [`Value::Array`].
/// Can be called either with a single [`Value::Array`] or variable list of Parameters.
///
/// # Errors
/// Returns an error if the [`Value::Array`] can not be sorted.
pub fn min(params: &[Value]) -> Result<Value, String> {
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
        .ok_or(String::from("function 'min' failed"))
}

/// Converts any [`Value`] to a [`Value::String`].
///
/// # Errors
/// Will return an error if not at least one parameter is supplied or if the [`Value`]
/// can not be converted.
pub fn str(params: &[Value]) -> Result<Value, String> {
    if let Some(value) = params.first() {
        Ok(Value::String(value.to_string()))
    } else {
        Err(String::from("no parameter supplied"))
    }
}

#[cfg(test)]
mod test {
    use super::{
        all, any, bool, compare, contains, empty, float, insert, int, length, max, min, str,
    };
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
