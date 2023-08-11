use crate::{error::Result, operator::Operator};
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
    /// # Errors
    /// Returns a [`SyntaxError`] when encountering invalid input.
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
        let previous = self.previous()?;
        match previous {
            Token::Literal(value) => Ok(Expression::Literal {
                value: value.clone(),
            }),
            Token::Identifier(name) => Ok(Expression::Variable { name: name.clone() }),
            Token::LeftParen => self.grouping(),
            Token::LeftBracket => self.array(),
            Token::Not | Token::Minus => self.unary(),
            _ => Err(SyntaxError::expected("left side of expression", previous)),
        }
    }

    fn do_infix(&mut self, left: Expression) -> Result<Expression> {
        match self.previous()? {
            Token::Minus
            | Token::Plus
            | Token::Star
            | Token::Slash
            | Token::Div
            | Token::Mod
            | Token::Equal
            | Token::NotEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Less
            | Token::LessEqual
            | Token::And
            | Token::Or
            | Token::Xor => self.binary(left),
            Token::LeftParen => self.call(left),
            _ => Err(SyntaxError("invalid infix Token".to_string())),
        }
    }

    fn expression_list(&mut self, end_token: &Token) -> Result<Vec<Expression>> {
        let mut expressions: Vec<Expression> = vec![];

        while self.current().is_some_and(|t| t != end_token) {
            expressions.push(self.expression()?);

            if self.current() == Some(&Token::Comma) {
                self.advance();
            }
        }

        self.chomp(end_token)?;

        Ok(expressions)
    }

    fn call(&mut self, left: Expression) -> Result<Expression> {
        if let Expression::Variable { name } = left {
            Ok(Expression::Call {
                name,
                params: self.expression_list(&Token::RightParen)?,
            })
        } else {
            Err(SyntaxError::expected("some identifier", self.previous()?))
        }
    }

    fn array(&mut self) -> Result<Expression> {
        Ok(Expression::Array {
            expressions: self.expression_list(&Token::RightBracket)?,
        })
    }

    fn binary(&mut self, left: Expression) -> Result<Expression> {
        let operator = Operator::try_from(self.previous()?)?;
        let right = self.parse_precedence(Precedence::from(self.previous()?).next())?;

        Ok(Expression::Binary {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        })
    }

    fn unary(&mut self) -> Result<Expression> {
        let operator = Operator::try_from(self.previous()?)?;
        let right = self.parse_precedence(Precedence::Unary)?;

        Ok(Expression::Unary {
            right: Box::new(right),
            operator,
        })
    }

    fn grouping(&mut self) -> Result<Expression> {
        let expression = self.expression()?;
        self.chomp(&Token::RightParen)?;

        Ok(expression)
    }

    fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Result<&Token> {
        self.tokens
            .get(self.current - 1)
            .ok_or(SyntaxError("expected some token".to_string()))
    }

    fn chomp(&mut self, token: &Token) -> Result<()> {
        if self.current() == Some(token) {
            self.advance();
            Ok(())
        } else {
            match self.current() {
                Some(current) => Err(SyntaxError(format!(
                    "Expected {token:?} encountered {current:?}"
                ))),
                None => Err(SyntaxError(format!(
                    "Expected {token:?} encountered end of file"
                ))),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression, error::SyntaxError, operator::Operator, token::Token, value::Value,
    };

    use super::Compiler;

    #[test]
    fn single_literal() {
        let ast = Compiler::compile_ast(vec![Token::Literal(Value::Boolean(true))]);
        let expected = Expression::Literal {
            value: Value::Boolean(true),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn single_variable() {
        let ast = Compiler::compile_ast(vec![Token::Identifier(String::from("test"))]);
        let expected = Expression::Variable {
            name: String::from("test"),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn expression_group() {
        let ast = Compiler::compile_ast(vec![
            Token::LeftParen,
            Token::Literal(Value::Boolean(true)),
            Token::RightParen,
        ]);
        let expected = Expression::Literal {
            value: Value::Boolean(true),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn unary_literal() {
        let ast = Compiler::compile_ast(vec![Token::Minus, Token::Literal(Value::Number(42.0))]);
        let expected = Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Number(42.0),
            }),
            operator: Operator::Minus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn multiply_number() {
        let ast = Compiler::compile_ast(vec![
            Token::Literal(Value::Number(3.0)),
            Token::Star,
            Token::Literal(Value::Number(2.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn add_number() {
        let ast = Compiler::compile_ast(vec![
            Token::Literal(Value::Number(3.0)),
            Token::Plus,
            Token::Literal(Value::Number(2.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn precedence_multiply_addition() {
        let ast = Compiler::compile_ast(vec![
            Token::Literal(Value::Number(1.0)),
            Token::Plus,
            Token::Literal(Value::Number(2.0)),
            Token::Star,
            Token::Literal(Value::Number(3.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal {
                    value: Value::Number(2.0),
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Number(3.0),
                }),
                operator: Operator::Multiply,
            }),
            operator: Operator::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn comparison_equal() {
        let ast = Compiler::compile_ast(vec![
            Token::Literal(Value::Number(5.0)),
            Token::Equal,
            Token::Literal(Value::Number(7.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(5.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(7.0),
            }),
            operator: Operator::Equal,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn boolean_and() {
        let ast = Compiler::compile_ast(vec![
            Token::Literal(Value::Boolean(true)),
            Token::And,
            Token::Literal(Value::Boolean(false)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::And,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_add() {
        let ast = Compiler::compile_ast(vec![
            Token::LeftParen,
            Token::Literal(Value::Number(5.0)),
            Token::Plus,
            Token::Identifier(String::from("SOME_VAR")),
            Token::RightParen,
            Token::Star,
            Token::Literal(Value::Number(4.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal {
                    value: Value::Number(5.0),
                }),
                right: Box::new(Expression::Variable {
                    name: String::from("SOME_VAR"),
                }),
                operator: Operator::Plus,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(4.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_mul() {
        let ast = Compiler::compile_ast(vec![
            Token::Identifier(String::from("SOME_VAR")),
            Token::Star,
            Token::Literal(Value::Number(4.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Variable {
                name: String::from("SOME_VAR"),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(4.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn function_call() {
        let ast = Compiler::compile_ast(vec![
            Token::Identifier(String::from("max")),
            Token::LeftParen,
            Token::Literal(Value::Number(1.0)),
            Token::Comma,
            Token::Literal(Value::Number(2.0)),
            Token::RightParen,
        ]);
        let expected = Expression::Call {
            name: String::from("max"),
            params: vec![
                Expression::Literal {
                    value: Value::Number(1.0),
                },
                Expression::Literal {
                    value: Value::Number(2.0),
                },
            ],
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn err_open_function_call() {
        let ast =
            Compiler::compile_ast(vec![Token::Identifier("max".to_string()), Token::LeftParen]);

        let expected = SyntaxError("Expected RightParen encountered end of file".to_string());

        assert_eq!(ast, Err(expected));
    }

    #[test]
    fn err_open_array() {
        let ast = Compiler::compile_ast(vec![Token::LeftBracket, Token::Literal(Value::Nil)]);

        let expected = SyntaxError("Expected RightBracket encountered end of file".to_string());
        assert_eq!(ast, Err(expected));
    }

    #[test]
    fn err_array_empty_expressions() {
        let ast =
            Compiler::compile_ast(vec![Token::LeftBracket, Token::Comma, Token::RightBracket]);

        let expected = SyntaxError("Expected left side of expression got \"Comma\"".to_string());
        assert_eq!(ast, Err(expected));
    }
}
