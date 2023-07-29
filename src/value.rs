use std::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    String(String),
    Number(f64),
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

impl Value {
    pub fn div_int(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Value::Number(lhs), Value::Number(rhs)) => Value::Number((lhs / rhs).trunc()),
            _ => Value::Nil,
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
