//! The **Simple Logic & Arithmetic Compiler** is a library to convert an single
//! expression statement into a structured [`Expression`] [abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree).
//!
//! The AST can be validated, (de)serialized, and executed using the built-in interpreter.
//!
//! # Example
//! ```
//! use slac::{check_variables_and_functions, compile, execute, StaticEnvironment, Value};
//! use slac::std::extend_environment;
//!
//! let ast = compile("max(10, 20) + 1").expect("compiles the ast");
//! let mut env = StaticEnvironment::default();
//!
//! extend_environment(&mut env);
//! check_variables_and_functions(&env, &ast).expect("find the usage of max");
//!
//! let result = execute(&env, &ast).expect("execute the expression");
//! assert_eq!(Value::Number(21.0), result);
//! ```
//!
//! # Serialization / Deserialization
//!
//! The [`Expression`] can be fully serialized into an (e.g.) JSON string for precompilation
//! and cached execution using [serde](https://crates.io/crates/serde). See `test/serde_test.rs`
//! for the resulting JSON.

mod ast;
mod compiler;
pub mod environment;
mod error;
mod interpreter;
mod operator;
mod scanner;
pub mod std;
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
pub use crate::validate::{check_boolean_result, check_variables_and_functions};
#[doc(inline)]
pub use crate::value::Value;

/// Compiles a string into an [`Expression`] tree.
///
/// # Errors
/// Returns an [`Error`] when encountering invalid Input.
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
///
/// # Remarks
/// * Currently uses an `TreeWalkingInterpreter` to evaluate the AST.
/// * Will [short-circuit](https://en.wikipedia.org/wiki/Short-circuit_evaluation) boolean expression.
/// * Invalid operations will be evaluated to [`Option::None`].
/// * Comparison of empty Values against [`Option::None`] is a valid operation
///   * e.g: `empty_var = ''` is valid
pub fn execute(env: &dyn Environment, ast: &Expression) -> Option<Value> {
    interpreter::TreeWalkingInterpreter::interprete(env, ast)
}
