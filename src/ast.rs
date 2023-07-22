use crate::token::Token;

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
    Literal(Token),
    /// A named variable, represented by a single [`Token`].
    Variable(Token),
    /// A call to an external function, named by a single [`Token`] and has a list
    /// of [`Expressions`](Expression) as parameters.
    Call(Token, Vec<Expression>),
}
