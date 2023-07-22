use crate::error::Result;
use std::vec;

use crate::{
    ast::Expression,
    error::SyntaxError,
    token::{Precedence, Token},
};

pub struct Compiler {
    tokens: Vec<Token>,
    current: usize,
}

impl Compiler {
    /// From a series of [`Tokens`](Token) compiles a structured [`Expression`] tree.
    pub fn compile_ast(tokens: Vec<Token>) -> Result<Expression> {
        let mut compiler = Compiler { tokens, current: 0 };
        compiler.compile()
    }

    fn compile(&mut self) -> Result<Expression> {
        let expression = self.expression()?;

        match self.current() {
            Some(token) => Err(SyntaxError::expected("end of expresssion", token)),
            None => Ok(expression),
        }
    }

    fn expression(&mut self) -> Result<Expression> {
        self.parse_precedence(Precedence::Or)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expression> {
        self.advance();
        let mut expression = self.do_prefix()?;

        while self
            .current()
            .is_some_and(|t| precedence <= Precedence::from(t))
        {
            self.advance();
            expression = self.do_infix(expression)?;
        }

        Ok(expression)
    }

    fn do_prefix(&mut self) -> Result<Expression> {
        let previous = self.previous();
        match previous {
            Token::Boolean(_) | Token::String(_) | Token::Number(_) => {
                Ok(Expression::Literal(previous.clone()))
            }
            Token::Identifier(_) => Ok(Expression::Variable(previous.clone())),
            Token::LeftParen => self.grouping(),
            Token::Not | Token::Minus => self.unary(),
            _ => Err(SyntaxError::expected("left side of expression", previous)),
        }
    }

    fn do_infix(&mut self, left: Expression) -> Result<Expression> {
        match self.previous() {
            Token::Minus
            | Token::Plus
            | Token::Star
            | Token::Slash
            | Token::Equal
            | Token::NotEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Less
            | Token::LessEqual
            | Token::And
            | Token::Or => self.binary(left),
            Token::LeftParen => self.call(left),
            _ => unreachable!(),
        }
    }

    fn arguments(&mut self) -> Result<Vec<Expression>> {
        let mut arguments: Vec<Expression> = vec![];

        if !(self.current() == Some(&Token::RightParen)) {
            loop {
                arguments.push(self.expression()?);

                if self.current() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }

            self.chomp(Token::RightParen, "')' after argument list")?;
        } else {
            self.advance(); // chomp the ')'
        }

        Ok(arguments)
    }

    fn call(&mut self, left: Expression) -> Result<Expression> {
        if let Expression::Variable(token) = left {
            Ok(Expression::Call(token, self.arguments()?))
        } else {
            Err(SyntaxError::expected("some identifier", self.previous()))
        }
    }

    fn binary(&mut self, left: Expression) -> Result<Expression> {
        let operator = self.previous().clone();
        let right = self.parse_precedence(Precedence::from(&operator).next())?;

        Ok(Expression::Binary {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        })
    }

    fn unary(&mut self) -> Result<Expression> {
        let operator = self.previous().clone();
        let right = self.parse_precedence(Precedence::Unary)?;

        Ok(Expression::Unary {
            right: Box::new(right),
            operator,
        })
    }

    fn grouping(&mut self) -> Result<Expression> {
        let expression = self.expression()?;
        self.chomp(Token::RightParen, "')' after group expression")?;

        Ok(expression)
    }

    fn advance(&mut self) {
        if self.current <= self.tokens.len() - 1 {
            self.current += 1;
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .expect("expected some token")
    }

    fn chomp(&mut self, ref token: Token, message: &str) -> Result<()> {
        if self.current() == Some(token) {
            self.advance();
            Ok(())
        } else {
            Err(SyntaxError::expected(message, token))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Expression, token::Token};

    use super::Compiler;

    #[test]
    fn single_literal() {
        let ast = Compiler::compile_ast(vec![Token::Boolean(true)]);
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn single_variable() {
        let ast = Compiler::compile_ast(vec![Token::Identifier(String::from("test"))]);
        let expected = Expression::Variable(Token::Identifier(String::from("test")));

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn expression_group() {
        let ast = Compiler::compile_ast(vec![
            Token::LeftParen,
            Token::Boolean(true),
            Token::RightParen,
        ]);
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn unary_literal() {
        let ast = Compiler::compile_ast(vec![Token::Minus, Token::Number(42.0)]);
        let expected = Expression::Unary {
            right: Box::new(Expression::Literal(Token::Number(42.0))),
            operator: Token::Minus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn multiply_number() {
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Star, Token::Number(2.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(3.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Star,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn add_number() {
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Plus, Token::Number(2.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(3.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn precedence_multiply_addition() {
        let ast = Compiler::compile_ast(vec![
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::Star,
            Token::Number(3.0),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(1.0))),
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Token::Number(2.0))),
                right: Box::new(Expression::Literal(Token::Number(3.0))),
                operator: Token::Star,
            }),
            operator: Token::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn comparison_equal() {
        let ast = Compiler::compile_ast(vec![Token::Number(5.0), Token::Equal, Token::Number(7.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(5.0))),
            right: Box::new(Expression::Literal(Token::Number(7.0))),
            operator: Token::Equal,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn boolean_and() {
        let ast = Compiler::compile_ast(vec![
            Token::Boolean(true),
            Token::And,
            Token::Boolean(false),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Boolean(true))),
            right: Box::new(Expression::Literal(Token::Boolean(false))),
            operator: Token::And,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_add() {
        let ast = Compiler::compile_ast(vec![
            Token::LeftParen,
            Token::Number(5.0),
            Token::Plus,
            Token::Identifier(String::from("SOME_VAR")),
            Token::RightParen,
            Token::Star,
            Token::Number(4.0),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(Token::Number(5.0))),
                right: Box::new(Expression::Variable(Token::Identifier(String::from(
                    "SOME_VAR",
                )))),
                operator: Token::Plus,
            }),
            right: Box::new(Expression::Literal(Token::Number(4.0))),
            operator: Token::Star,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_mul() {
        let ast = Compiler::compile_ast(vec![
            Token::Identifier(String::from("SOME_VAR")),
            Token::Star,
            Token::Number(4.0),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Variable(Token::Identifier(String::from(
                "SOME_VAR",
            )))),
            right: Box::new(Expression::Literal(Token::Number(4.0))),
            operator: Token::Star,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn function_call() {
        let ast = Compiler::compile_ast(vec![
            Token::Identifier(String::from("max")),
            Token::LeftParen,
            Token::Number(1.0),
            Token::Comma,
            Token::Number(2.0),
            Token::RightParen,
        ]);
        let expected = Expression::Call(
            Token::Identifier(String::from("max")),
            vec![
                Expression::Literal(Token::Number(1.0)),
                Expression::Literal(Token::Number(2.0)),
            ],
        );

        assert_eq!(ast, Ok(expected));
    }
}
