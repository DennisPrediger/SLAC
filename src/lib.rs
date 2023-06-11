use ast::Expression;
use compiler::Compiler;
use scanner::Scanner;

pub mod ast;
pub mod compiler;
pub mod scanner;
pub mod token;

pub fn compile(source: &str) -> Expression {
    let tokens = Scanner::tokenize(source);
    Compiler::compile_ast(tokens)
}
