//! The **Simple Logic & Arithmetic Compiler** is a library to convert an expression
//! string into a structured [`ast::Expression`] tree (an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree)).
//!
//! While the interals are public you may want to use the [`compile`] function
//! which returns a [`Result`] of either the compiled [`ast::Expression`]
//! tree or an [`error::SyntaxError`].

use ast::Expression;
use compiler::Compiler;
use error::SyntaxError;
use scanner::Scanner;

pub mod ast;
pub mod compiler;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod scanner;
pub mod token;
pub mod value;

/// Compiles a string into an [`ast::Expression`] tree or an [`error::SyntaxError`].
///
/// # Examples
/// ```
/// use slac::{ast::Expression, compile, token::Token, value::Value};
///
/// let ast = compile("10 + 20 >= 30");
/// let expected = Expression::Binary {
///     left: Box::new(Expression::Binary {
///         left: Box::new(Expression::Literal(Value::Number(10.0))),
///         right: Box::new(Expression::Literal(Value::Number(20.0))),
///         operator: Token::Plus,
///     }),
///     right: Box::new(Expression::Literal(Value::Number(30.0))),
///     operator: Token::GreaterEqual,
/// };
///
/// assert_eq!(ast, Ok(expected));
/// ```
pub fn compile(source: &str) -> Result<Expression, SyntaxError> {
    let tokens = Scanner::tokenize(source)?;
    let ast = Compiler::compile_ast(tokens)?;

    Ok(ast)
}
