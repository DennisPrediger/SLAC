//! The **Simple Logic & Arithmetic Compiler** is a library to convert an expression
//! string into a structured [`ast::Expression`] tree (an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree)).
//!
//! While the interals are public you may want to use the [`compile`] function
//! which returns a [`Result`] of either the compiled [`ast::Expression`]
//! tree or an [`error::SyntaxError`].
//!
//! The [`AST`](ast::Expression) can be evaluated using the built-in [`interpreter::TreeWalkingInterpreter`].

mod ast;
mod compiler;
pub mod environment;
mod error;
mod interpreter;
mod operator;
mod scanner;
pub mod stdlib;
mod token;
mod validate;
mod value;

use crate::environment::Environment;

#[doc(inline)]
pub use crate::ast::Expression;
#[doc(inline)]
pub use crate::compiler::Compiler;
#[doc(inline)]
pub use crate::environment::StaticEnvironment;
#[doc(inline)]
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::operator::Operator;
#[doc(inline)]
pub use crate::scanner::Scanner;
#[doc(inline)]
pub use crate::token::Token;
#[doc(inline)]
pub use crate::validate::{validate_boolean_result, validate_env};
#[doc(inline)]
pub use crate::value::Value;

/// Compiles a string into an [`ast::Expression`] tree.
///
/// # Errors
/// Returns a [`error::SyntaxError`] when encountering invalid Input.
///
/// # Examples
/// ```
/// use slac::{compile, Expression, Operator, Token, Value};
/// let ast = compile("10 + 20 >= 30");
/// let expected = Expression::Binary {
///     left: Box::new(Expression::Binary {
///         left: Box::new(Expression::Literal {
///             value : Value::Number(10.0)
///         }),
///         right: Box::new(Expression::Literal {
///             value : Value::Number(20.0)
///         }),
///         operator: Operator::Plus,
///     }),
///     right: Box::new(Expression::Literal {
///         value : Value::Number(30.0)
///     }),
///     operator: Operator::GreaterEqual,
/// };
///
/// assert_eq!(ast, Ok(expected));
/// ```
pub fn compile(source: &str) -> Result<Expression> {
    let tokens = Scanner::tokenize(source)?;
    let ast = Compiler::compile_ast(tokens)?;

    Ok(ast)
}

/// Executes an [`Expression`] using an [`Environment`].
///
/// # Example
/// ```
/// use slac::{Expression, Operator, Value};
/// use slac::{execute, StaticEnvironment};
///
/// let env = StaticEnvironment::default();
/// let ast = Expression::Binary {
///     left: Box::new(Expression::Literal {
///         value: Value::Number(40.0),
///     }),
///     right: Box::new(Expression::Literal {
///         value: Value::Number(2.0),
///     }),
///     operator: Operator::Plus,
/// };
///
/// assert_eq!(Some(Value::Number(42.0)), execute(&env, &ast));
/// ```
pub fn execute(env: &dyn Environment, ast: &Expression) -> Option<Value> {
    interpreter::TreeWalkingInterpreter::interprete(env, ast)
}
