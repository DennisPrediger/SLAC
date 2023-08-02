#[cfg(feature = "serde")]
use serde;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "serde")]
use serde_json::json;

use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
};

/// A value used in the [`TreeWalkingInterpreter`](crate::interpreter::TreeWalkingInterpreter).
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    /// [`Value::Nil`] is only created by invalid operations and not from literals
    /// in the AST.
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Value>),
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(value) => Value::Number(-value),
            _ => Value::Nil,
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(value) => Value::Boolean(!value),
            _ => Value::Nil,
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::String(lhs), Value::String(rhs)) => Value::String(lhs + &rhs),
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
            (Value::Array(lhs), Value::Array(rhs)) => Value::Array([lhs, rhs].concat()),
            _ => Value::Nil,
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),
            _ => Value::Nil,
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),
            _ => Value::Nil,
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),
            _ => Value::Nil,
        }
    }
}

impl Rem for Value {
    type Output = Value;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs % rhs),
            _ => Value::Nil,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Array(v) => write!(f, "{:?}", v),
        }
    }
}

impl Value {
    /// Integer division between two operands. Returns the whole number quotient,
    /// discarding any fractional part.
    ///
    /// # Examples
    /// ```
    /// use slac::value::Value;
    ///
    /// let a = Value::Number(10.0);
    /// let b = Value::Number(3.0);
    ///
    /// assert_eq!(Value::Number(3.0), a.div_int(b));
    /// ```
    pub fn div_int(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number((lhs / rhs).trunc()),
            _ => Value::Nil,
        }
    }

    /// Converts a [`Value`] into a [`serde_json::Value`].
    #[cfg(feature = "serde")]
    pub fn as_json(self) -> serde_json::Value {
        match self {
            Value::Nil => json!(null),
            Value::Boolean(v) => json!(v),
            Value::String(v) => json!(v),
            Value::Number(v) => json!(v),
            Value::Array(v) => {
                json!(v
                    .into_iter()
                    .map(Value::as_json)
                    .collect::<Vec<serde_json::Value>>())
            }
        }
    }
}

/// Converts a [`serde_json::Value`] into a [`Value`].
#[cfg(feature = "serde")]
impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Nil,
            serde_json::Value::Bool(v) => Value::Boolean(v),
            serde_json::Value::Number(v) => Value::Number(v.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(v) => Value::String(v),
            serde_json::Value::Array(v) => Value::Array(v.into_iter().map(Value::from).collect()),
            serde_json::Value::Object(_) => Value::Nil,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Value;

    fn test_div_int(divisor: f64) -> Value {
        let a = Value::Number(10.0);
        let b = Value::Number(divisor);

        a.div_int(b)
    }

    #[test]
    fn number_div() {
        assert_eq!(Value::Number(f64::INFINITY), test_div_int(0.0));
        assert_eq!(Value::Number(10.0), test_div_int(1.0));
        assert_eq!(Value::Number(5.0), test_div_int(2.0));
        assert_eq!(Value::Number(3.0), test_div_int(3.0));
        assert_eq!(Value::Number(2.0), test_div_int(4.0));
        assert_eq!(Value::Number(2.0), test_div_int(5.0));
        assert_eq!(Value::Number(1.0), test_div_int(6.0));
        assert_eq!(Value::Number(1.0), test_div_int(7.0));
        assert_eq!(Value::Number(1.0), test_div_int(9.0));
        assert_eq!(Value::Number(1.0), test_div_int(10.0));
        assert_eq!(Value::Number(0.0), test_div_int(11.0));
    }

    fn test_mod_int(divisor: f64) -> Value {
        let a = Value::Number(10.0);
        let b = Value::Number(divisor);

        a % b
    }

    #[test]
    fn number_mod() {
        match test_mod_int(0.0) {
            Value::Number(value) => assert!(value.is_nan()),
            _ => panic!(),
        }
        assert_eq!(Value::Number(0.0), test_mod_int(1.0));
        assert_eq!(Value::Number(0.0), test_mod_int(2.0));
        assert_eq!(Value::Number(1.0), test_mod_int(3.0));
        assert_eq!(Value::Number(2.0), test_mod_int(4.0));
        assert_eq!(Value::Number(0.0), test_mod_int(5.0));
        assert_eq!(Value::Number(4.0), test_mod_int(6.0));
        assert_eq!(Value::Number(3.0), test_mod_int(7.0));
        assert_eq!(Value::Number(2.0), test_mod_int(8.0));
        assert_eq!(Value::Number(1.0), test_mod_int(9.0));
        assert_eq!(Value::Number(0.0), test_mod_int(10.0));
    }
}

#[cfg(all(test, feature = "serde"))]
mod test_serde_json {
    use serde_json::json;

    use crate::value::Value;

    #[test]
    fn convert_from_json() {
        assert_eq!(Value::Nil, Value::from(json!(null)));
        assert_eq!(Value::Boolean(true), Value::from(json!(true)));
        assert_eq!(Value::String("abc".to_string()), Value::from(json!("abc")));
        assert_eq!(Value::Number(19.9), Value::from(json!(19.9)));
    }

    #[test]
    fn convert_to_json() {
        assert_eq!(json!(null), Value::Nil.as_json());
        assert_eq!(json!(true), Value::Boolean(true).as_json());
        assert_eq!(json!("abc"), Value::String("abc".to_string()).as_json());
        assert_eq!(json!(19.9), Value::Number(19.9).as_json());
    }
}
