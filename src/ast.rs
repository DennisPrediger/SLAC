use crate::{token::Token, value::Value};

/// An expression represents an entity which can be evaluated to a value.
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Expression {
    /// An operation on a single [`Expression`] operand with a [`Token`] operator.
    Unary {
        right: Box<Expression>,
        operator: Token,
    },
    /// An operation on two [`Expression`] operands with a [`Token`] operator.
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: Token,
    },
    /// A literal Value, represented by a single [`Token`].
    Literal(Value),
    /// A named external variable.
    Variable(String),
    /// A call to an external function with a list of [`Expression`] parameters.
    Call(String, Vec<Expression>),
}
