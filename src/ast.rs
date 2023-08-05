#[cfg(feature = "serde")]
use serde;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::operator::Operator;
use crate::value::Value;

/// An expression represents an entity which can be evaluated to a value.
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Expression {
    /// An operation on a single [`Expression`] operand with an [`Operator`]
    Unary {
        right: Box<Expression>,
        operator: Operator,
    },
    /// An operation on two [`Expression`] operands with a an [`Operator`].
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: Operator,
    },
    /// An list of [`Expression`] values.
    Array { expressions: Vec<Expression> },
    /// A [`Value`] literal.
    Literal { value: Value },
    /// A named external variable.
    Variable { name: String },
    /// A call to an external function with a list of [`Expression`] parameters.
    Call {
        name: String,
        params: Vec<Expression>,
    },
}
