use ast::Expression;
use compiler::Compiler;
use error::SyntaxError;
use scanner::Scanner;

pub mod ast;
pub mod compiler;
pub mod error;
pub mod scanner;
pub mod token;

/// Compiles an string expression into into a structured AST
///
/// # Examples
///
/// ```
/// use slac::{ast::Expression, compile, token::Token};
///
/// let ast = compile("10 + 20 >= 30");
/// let expected = Expression::Binary {
///     left: Box::new(Expression::Binary {
///         left: Box::new(Expression::Literal(Token::Number(10.0))),
///         right: Box::new(Expression::Literal(Token::Number(20.0))),
///         operator: Token::Plus,
///     }),
///     right: Box::new(Expression::Literal(Token::Number(30.0))),
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
