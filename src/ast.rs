#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::operator::Operator;
use crate::value::Value;

/// An `Expression` is a statement which can always be evaluated to a single [`Value`].
/// A recursive `Expression` is the foundation of an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree).
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(tag = "type", rename_all = "camelCase")
)]
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Expression {
    /// An unary operation on a single `Expression` operand using an [`Operator`]
    Unary {
        right: Box<Expression>,
        operator: Operator,
    },
    /// An binary operation on two `Expression` operands using an [`Operator`].
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: Operator,
    },
    Ternary {
        left: Box<Expression>,
        middle: Box<Expression>,
        right: Box<Expression>,
        operator: Operator,
    },
    /// An list of not yet evaluated `Expression` values.
    Array { expressions: Vec<Expression> },
    /// A literal [`Value`].
    Literal { value: Value },
    /// A named external variable.
    Variable { name: String },
    /// A call to an external function with a list of not yet evaluated `Expression` parameters.
    Call {
        name: String,
        params: Vec<Expression>,
    },
}
