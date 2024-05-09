#[cfg(feature = "serde")]
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use std::{
    cmp::Ordering,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::{Add, BitXor, Div, Mul, Neg, Not, Rem, Sub},
};

use crate::{
    error::{self, Error},
    Operator,
};

/// A Wrapper for the four different possible variable types.
#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Value>),
}
impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // direct comparision of contained types
            (Value::Boolean(l0), Value::Boolean(r0)) => l0.cmp(r0),
            (Value::String(l0), Value::String(r0)) => l0.cmp(r0),
            (Value::Number(l0), Value::Number(r0)) => l0.total_cmp(r0), // total_cmp for f64
            (Value::Array(l0), Value::Array(r0)) => l0.cmp(r0),

            // comparison by ordinal value
            (left, right) => left.ordinal().cmp(&right.ordinal()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Neg for Value {
    type Output = error::Result<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(value) => Ok(Value::Number(-value)),
            _ => Err(Error::InvalidUnaryOperator(Operator::Minus)),
        }
    }
}

impl Not for Value {
    type Output = error::Result<Value>;

    fn not(self) -> Self::Output {
        Ok(Value::Boolean(!self.as_bool()))
    }
}

impl Add for Value {
    type Output = error::Result<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::String(lhs), Value::String(rhs)) => Ok(Value::String(lhs + &rhs)),
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
            (Value::Array(lhs), Value::Array(rhs)) => Ok(Value::Array([lhs, rhs].concat())),
            _ => Err(Error::InvalidBinaryOperator(Operator::Plus)),
        }
    }
}

impl Sub for Value {
    type Output = error::Result<Value>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),
            _ => Err(Error::InvalidBinaryOperator(Operator::Minus)),
        }
    }
}

impl Mul for Value {
    type Output = error::Result<Value>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),
            _ => Err(Error::InvalidBinaryOperator(Operator::Multiply)),
        }
    }
}

impl Div for Value {
    type Output = error::Result<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),
            _ => Err(Error::InvalidBinaryOperator(Operator::Divide)),
        }
    }
}

impl Rem for Value {
    type Output = error::Result<Value>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs % rhs)),
            _ => Err(Error::InvalidBinaryOperator(Operator::Mod)),
        }
    }
}

impl BitXor for Value {
    type Output = error::Result<Value>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs ^ rhs)),
            _ => Err(Error::InvalidBinaryOperator(Operator::Xor)),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    /// assert_eq!(Ok(Value::Number(3.0)), a.div_int(b));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBinaryOperator`] if any side of the operator is not a Number.
    pub fn div_int(self, rhs: Self) -> error::Result<Self> {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number((lhs / rhs).trunc())),
            _ => Err(Error::InvalidBinaryOperator(Operator::Div)),
        }
    }

    /// Returns the length of a `String` or `Array` `Value`.
    /// `Boolean` and `Number` have a length of 0.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Value::String(v) => v.len(),
            Value::Array(v) => v.len(),
            _ => 0,
        }
    }

    /// Checks if the value is equal to the result of [`Value::empty()`].
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Value::empty(self)
    }

    /// Returns an new empty `Value` of the same type as the callee.
    /// * `Value::Boolean` -> `false`
    /// * `Value::String` -> `''`
    /// * `Value::Number` -> `0`
    /// * `Value::Array` -> `[]`
    #[must_use]
    pub fn empty(&self) -> Self {
        match self {
            Value::Boolean(_) => Value::Boolean(false),
            Value::String(_) => Value::String(String::new()),
            Value::Number(_) => Value::Number(0.0),
            Value::Array(_) => Value::Array(vec![]),
        }
    }

    /// Returns the boolean representation of the `Value`.
    /// Returns Booleans _as is_. Other `Value` kinds are based on
    /// if the contained value is not [`Value::is_empty()`].
    #[must_use]
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(v) => *v,
            value => !value.is_empty(),
        }
    }

    /// Returns an ordinal value for each [`Value`] kind.
    #[must_use]
    fn ordinal(&self) -> u8 {
        match self {
            Value::Boolean(_) => 0,
            Value::String(_) => 1,
            Value::Number(_) => 2,
            Value::Array(_) => 3,
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
    use crate::{Error, Operator};

    use super::Value;

    fn test_div_int(divisor: f64) -> Value {
        let a = Value::Number(10.0);
        let b = Value::Number(divisor);

        a.div_int(b).unwrap()
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

        (a % b).unwrap()
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
    fn is_empty() {
        assert_eq!(true, Value::Boolean(false).is_empty());
        assert_eq!(false, Value::Boolean(true).is_empty());
        assert_eq!(true, Value::Number(0.0).is_empty());
        assert_eq!(false, Value::Number(1.0).is_empty());

        assert_eq!(true, Value::String(String::new()).is_empty());
        assert_eq!(false, Value::String(String::from("something")).is_empty());

        assert_eq!(true, Value::Array(vec![]).is_empty());
        assert_eq!(false, Value::Array(vec![Value::Boolean(true)]).is_empty());
    }

    #[test]
    fn invalid_operations() {
        assert_eq!(
            Err(Error::InvalidUnaryOperator(Operator::Minus)),
            -Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Plus)),
            Value::Number(10.0) + Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Minus)),
            Value::Number(10.0) - Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Multiply)),
            Value::Number(10.0) * Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Divide)),
            Value::Number(10.0) / Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Mod)),
            Value::Number(10.0) % Value::String(String::from("a string"))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Div)),
            Value::Number(10.0).div_int(Value::Boolean(false))
        );
        assert_eq!(
            Err(Error::InvalidBinaryOperator(Operator::Xor)),
            Value::Number(10.0) ^ Value::Boolean(false)
        );
    }
}

#[cfg(all(test, feature = "serde"))]
mod test_serde_json {
    use std::vec;

    use serde_json::json;

    use crate::value::Value;

    #[test]
    fn convert_from_json() {
        assert!(serde_json::from_value::<Value>(json!(null)).is_err());

        assert_eq!(
            Value::Boolean(true),
            serde_json::from_value(json!(true)).unwrap()
        );
        assert_eq!(
            Value::String(String::from("ab")),
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
        assert_eq!(json!(true), json!(Value::Boolean(true)));
        assert_eq!(
            json!(String::from("ab")),
            json!(Value::String(String::from("ab")))
        );
        assert_eq!(json!(19.9), json!(Value::Number(19.9)));
        assert_eq!(
            json!(["hallo", 42.0, false]),
            json!(Value::Array(vec![
                Value::String(String::from("hallo")),
                Value::Number(42.0),
                Value::Boolean(false)
            ]))
        );
    }
}
