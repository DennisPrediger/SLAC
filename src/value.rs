#[cfg(feature = "serde")]
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use std::{
    fmt::Display,
    ops::{Add, BitXor, Div, Mul, Neg, Not, Rem, Sub},
};

/// A value used in the [`TreeWalkingInterpreter`](crate::interpreter::TreeWalkingInterpreter).
#[derive(Debug, PartialOrd, Clone)]
pub enum Value {
    /// [`Value::Nil`] is only created by invalid operations and not from literals
    /// in the AST.
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::Nil, Self::Boolean(r0)) => r0 == &false,
            (Self::Nil, Self::String(r0)) => r0.is_empty(),
            (Self::Nil, Self::Number(r0)) => r0 == &0.0,
            (Self::Nil, Self::Array(r0)) => r0.is_empty(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
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

impl BitXor for Value {
    type Output = Value;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Value::Boolean(lhs ^ rhs),
            _ => Value::Nil,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Boolean(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Array(v) => write!(f, "{v:?}"),
        }
    }
}

impl Value {
    /// Integer division between two operands. Returns the whole number quotient,
    /// discarding any fractional part.
    ///
    /// # Examples
    /// ```
    /// use slac::Value;
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

    pub fn len(&self) -> usize {
        match self {
            Value::String(v) => v.len(),
            Value::Array(v) => v.len(),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Value::String(v) => v.is_empty(),
            Value::Array(v) => v.is_empty(),
            _ => false,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Nil => serializer.serialize_unit(),
            Value::Boolean(v) => serializer.serialize_bool(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Number(v) => serializer.serialize_f64(*v),
            Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for element in v {
                    seq.serialize_element(element)?;
                }
                seq.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(feature = "serde")]
struct ValueVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a primitive value or list")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Nil)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Boolean(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Number(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut values = vec![];
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }

        Ok(Value::Array(values))
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

    #[test]
    fn compare_empty_values() {
        assert_eq!(Value::Nil, Value::Boolean(false));
        assert_ne!(Value::Nil, Value::Boolean(true));

        assert_eq!(Value::Nil, Value::String("".to_string()));
        assert_ne!(Value::Nil, Value::String("0".to_string()));
        assert_ne!(Value::Nil, Value::String("123".to_string()));

        assert_eq!(Value::Nil, Value::Number(0.0));
        assert_eq!(Value::Nil, Value::Number(-0.0));
        assert_ne!(Value::Nil, Value::Number(1.0));
        assert_ne!(Value::Nil, Value::Number(-100.0));

        assert_eq!(Value::Nil, Value::Array(vec![]));
        assert_ne!(Value::Nil, Value::Array(vec![Value::Nil]));
    }

    #[test]
    fn is_empty() {
        assert_eq!(false, Value::Nil.is_empty());
        assert_eq!(false, Value::Boolean(false).is_empty());
        assert_eq!(false, Value::Number(0.0).is_empty());

        assert_eq!(true, Value::String(String::new()).is_empty());
        assert_eq!(false, Value::String(String::from("something")).is_empty());

        assert_eq!(true, Value::Array(vec![]).is_empty());
        assert_eq!(false, Value::Array(vec![Value::Nil]).is_empty());
    }

    #[test]
    fn invalid_operations() {
        assert_eq!(Value::Nil, -Value::String("a string".to_string()));
        assert_eq!(Value::Nil, !Value::String("a string".to_string()));

        assert_eq!(
            Value::Nil,
            Value::Number(10.0) + Value::String("a string".to_string())
        );
        assert_eq!(
            Value::Nil,
            Value::Number(10.0) - Value::String("a string".to_string())
        );
        assert_eq!(
            Value::Nil,
            Value::Number(10.0) * Value::String("a string".to_string())
        );
        assert_eq!(
            Value::Nil,
            Value::Number(10.0) / Value::String("a string".to_string())
        );
        assert_eq!(
            Value::Nil,
            Value::Number(10.0) % Value::String("a string".to_string())
        );
        assert_eq!(
            Value::Nil,
            Value::Number(10.0).div_int(Value::Boolean(false))
        );
        assert_eq!(Value::Nil, Value::Number(10.0) ^ Value::Boolean(false));
    }
}

#[cfg(all(test, feature = "serde"))]
mod test_serde_json {
    use std::vec;

    use serde_json::json;

    use crate::value::Value;

    #[test]
    fn convert_from_json() {
        assert_eq!(Value::Nil, serde_json::from_value(json!(null)).unwrap());
        assert_eq!(
            Value::Boolean(true),
            serde_json::from_value(json!(true)).unwrap()
        );
        assert_eq!(
            Value::String("ab".to_string()),
            serde_json::from_value(json!("ab")).unwrap()
        );
        assert_eq!(
            Value::Number(19.9),
            serde_json::from_value(json!(19.9)).unwrap()
        );
        assert_eq!(
            Value::Array(vec![Value::Boolean(true), Value::Boolean(false)]),
            serde_json::from_value(json!(vec![true, false])).unwrap()
        );
    }

    #[test]
    fn convert_to_json() {
        assert_eq!(json!(null), json!(Value::Nil));
        assert_eq!(json!(true), json!(Value::Boolean(true)));
        assert_eq!(json!("ab"), json!(Value::String("ab".to_string())));
        assert_eq!(json!(19.9), json!(Value::Number(19.9)));
        assert_eq!(
            json!(["hallo", 42.0, false]),
            json!(Value::Array(vec![
                Value::String("hallo".to_string()),
                Value::Number(42.0),
                Value::Boolean(false)
            ]))
        );
    }
}
