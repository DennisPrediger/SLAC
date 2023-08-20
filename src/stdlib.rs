use std::cmp::Ordering;

use crate::{StaticEnvironment, Value};

pub fn add_stdlib(env: &mut StaticEnvironment) {
    env.add_native_func("abs", Some(1), abs);
    env.add_native_func("all", None, all);
    env.add_native_func("any", None, any);
    env.add_native_func("bool", Some(1), bool);
    env.add_native_func("contains", Some(2), contains);
    env.add_native_func("empty", Some(1), empty);
    env.add_native_func("float", Some(1), float);
    env.add_native_func("int", Some(1), int);
    env.add_native_func("length", Some(1), length);
    env.add_native_func("lowercase", Some(1), lowercase);
    env.add_native_func("uppercase", Some(1), uppercase);
    env.add_native_func("max", None, max);
    env.add_native_func("min", None, min);
    env.add_native_func("pow", None, pow);
    env.add_native_func("round", Some(1), round);
    env.add_native_func("str", Some(1), str);
    env.add_native_func("trim", Some(1), trim);

    env.add_var("pi", Value::Number(std::f64::consts::PI));
    env.add_var("e", Value::Number(std::f64::consts::E));
    env.add_var("tau", Value::Number(std::f64::consts::TAU));
}

fn smart_vec(params: &[Value]) -> &[Value] {
    match params.first() {
        Some(Value::Array(v)) if (params.len() == 1) => v, // only one Array parameter
        _ => params,                                       // all varadic params
    }
}

pub fn abs(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::Number(value)) = params.first() {
        Ok(Value::Number(value.abs()))
    } else {
        Err("not enough parameters".to_string())
    }
}

pub fn all(params: &[Value]) -> Result<Value, String> {
    let values = smart_vec(params);
    let result = values.iter().all(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

pub fn any(params: &[Value]) -> Result<Value, String> {
    let values = smart_vec(params);
    let result = values.iter().any(|v| v == &Value::Boolean(true));

    Ok(Value::Boolean(result))
}

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
        None => Err("not enough parameters".to_string()),
    }
}

pub fn contains(params: &[Value]) -> Result<Value, String> {
    let found = match (params.get(0), params.get(1)) {
        (Some(haystack), Some(needle)) => match (haystack, needle) {
            (Value::String(needle), Value::String(haystack)) => needle.contains(haystack), // search in String
            (Value::Array(haystack), needle) => haystack.iter().any(|v| v == needle), // search in Array
            _ => return Err("param types invalid".to_string()),
        },
        _ => return Err("not enough parameters".to_string()),
    };

    Ok(Value::Boolean(found))
}

pub fn empty(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => Ok(Value::Boolean(value.is_empty())),
        None => Err("no parameter supplied".to_string()),
    }
}

pub fn float(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => match value {
            Value::String(v) => {
                let float = v.parse::<f64>().map_err(|e| e.to_string())?;
                Ok(Value::Number(float))
            }
            Value::Number(_) => Ok(value.clone()),
            _ => Err("value can not be converted to float".to_string()),
        },
        None => Err("not enough parameters".to_string()),
    }
}

pub fn int(params: &[Value]) -> Result<Value, String> {
    if let Value::Number(value) = float(params)? {
        Ok(Value::Number(value.trunc()))
    } else {
        Err("undefined input value".to_string())
    }
}

pub fn length(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(value) => Ok(Value::Number(value.len() as f64)),
        None => Err("no parameter supplied".to_string()),
    }
}

pub fn lowercase(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.to_lowercase()))
    } else {
        Err("no param supplied".to_string())
    }
}

pub fn uppercase(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.to_uppercase()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

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
        .ok_or("function 'max' failed".to_string())
}

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
        .ok_or("function 'min' failed".to_string())
}

pub fn pow(params: &[Value]) -> Result<Value, String> {
    match (params.get(0), params.get(1)) {
        (Some(Value::Number(base)), exp) => {
            let exp = match exp {
                Some(Value::Number(exp)) => *exp,
                _ => 2.0,
            };
            Ok(Value::Number(base.powf(exp)))
        }
        _ => Err("not enough parameters".to_string()),
    }
}

pub fn round(params: &[Value]) -> Result<Value, String> {
    match params.first() {
        Some(Value::Number(v)) => Ok(Value::Number(v.round())),
        _ => Err("no parameter supplied".to_string()),
    }
}

pub fn str(params: &[Value]) -> Result<Value, String> {
    if let Some(value) = params.first() {
        Ok(Value::String(value.to_string()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

pub fn trim(params: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(value)) = params.first() {
        Ok(Value::String(value.trim().to_string()))
    } else {
        Err("no parameter supplied".to_string())
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::{
        stdlib::{
            abs, all, any, bool, contains, empty, float, int, length, lowercase, max, min, pow,
            round, str, trim, uppercase,
        },
        value::Value,
    };

    #[test]
    fn std_abs() {
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(10.0)]));
        assert_eq!(Ok(Value::Number(10.0)), abs(&vec![Value::Number(-10.0)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(12.34)]));
        assert_eq!(Ok(Value::Number(12.34)), abs(&vec![Value::Number(-12.34)]));

        assert!(abs(&vec![Value::String("-12.34".to_string())]).is_err());
    }

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
            bool(&vec![Value::String("true".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            bool(&vec![Value::String("other".to_string())]).unwrap()
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
                Value::String("Hello World".to_string()),
                Value::String("World".to_string())
            ])
        );

        assert_eq!(
            Ok(Value::Boolean(false)),
            contains(&vec![
                Value::String("Hello World".to_string()),
                Value::String("WORLD".to_string())
            ])
        );

        assert!(min(&vec![]).is_err());
    }

    #[test]
    fn std_empty() {
        assert_eq!(
            Value::Boolean(true),
            empty(&vec![Value::String("".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Boolean(false),
            empty(&vec![Value::String("🙄".to_string())]).unwrap()
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
            float(&vec![Value::String("12.2".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Number(-12.2),
            float(&vec![Value::String("-12.2".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Number(0.123),
            float(&vec![Value::String(".123".to_string())]).unwrap()
        );

        assert!(float(&vec![]).is_err());
        assert!(float(&vec![Value::Boolean(false)]).is_err());
    }

    #[test]
    fn std_int() {
        assert_eq!(
            Value::Number(12.0),
            int(&vec![Value::String("12.2".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Number(-12.0),
            int(&vec![Value::String("-12.2".to_string())]).unwrap()
        );

        assert_eq!(
            Value::Number(0.0),
            int(&vec![Value::String(".123".to_string())]).unwrap()
        );

        assert!(int(&vec![]).is_err());
        assert!(int(&vec![Value::Boolean(false)]).is_err());
    }

    #[test]
    fn std_length() {
        assert_eq!(Ok(Value::Number(0.0)), length(&vec![Value::Boolean(true)]));
        assert_eq!(Ok(Value::Number(0.0)), length(&vec![Value::Number(100.0)]));

        assert_eq!(
            Ok(Value::Number(5.0)),
            length(&vec![Value::String("Hello".to_string())])
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
    fn std_lowercase() {
        assert_eq!(
            Ok(Value::String("hello world".to_string())),
            lowercase(&vec![Value::String("Hello World".to_string())])
        );

        assert!(lowercase(&vec![]).is_err());
        assert!(lowercase(&vec![Value::Boolean(true)]).is_err());
    }

    #[test]
    fn std_uppercase() {
        assert_eq!(
            Ok(Value::String("HELLO WORLD".to_string())),
            uppercase(&vec![Value::String("Hello World".to_string())])
        );

        assert!(uppercase(&vec![]).is_err());
        assert!(uppercase(&vec![Value::Boolean(true)]).is_err());
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
    fn std_pow() {
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
    fn std_round() {
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
    fn std_str() {
        assert_eq!(
            Ok(Value::String("123".to_string())),
            str(&vec![Value::String("123".to_string())])
        );

        assert_eq!(
            Ok(Value::String("123".to_string())),
            str(&vec![Value::Number(123.0)])
        );

        assert_eq!(
            Ok(Value::String("true".to_string())),
            str(&vec![Value::Boolean(true)])
        );

        assert!(str(&vec![]).is_err());
    }

    #[test]
    fn std_trim() {
        assert_eq!(
            Ok(Value::String("Hello World".to_string())),
            trim(&vec![Value::String("  Hello World       ".to_string())])
        );

        assert!(trim(&vec![]).is_err());
        assert!(trim(&vec![Value::Boolean(true)]).is_err());
    }
}
