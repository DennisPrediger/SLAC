use crate::{
    ast::Expression,
    token::{Precedence, Token},
};

pub struct Compiler {
    tokens: Vec<Token>,
    current: usize,
}

impl Compiler {
    pub fn compile_ast(tokens: Vec<Token>) -> Expression {
        let mut compiler = Compiler { tokens, current: 0 };

        match compiler.compile() {
            Ok(expression) => expression,
            Err(message) => Expression::Error(message),
        }
    }

    fn compile(&mut self) -> Result<Expression, String> {
        let expression = self.expression()?;

        if self.current().is_some() {
            Err("only one expression expected".to_string())
        } else {
            Ok(expression)
        }
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.parse_precedence(Precedence::Or)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expression, String> {
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

    fn do_prefix(&mut self) -> Result<Expression, String> {
        let previous = self.previous();
        match previous {
            Token::Boolean(_) | Token::String(_) | Token::Number(_) => {
                Ok(Expression::Literal(previous.clone()))
            }
            Token::Identifier(_) => Ok(Expression::Variable(previous.clone())),
            Token::LeftParen => self.grouping(),
            Token::Not | Token::Minus => self.unary(),
            _ => Err("expected expression".to_string()),
        }
    }

    fn do_infix(&mut self, left: Expression) -> Result<Expression, String> {
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
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, left: Expression) -> Result<Expression, String> {
        let operator = self.previous().clone();
        let right = self.parse_precedence(Precedence::from(&operator).next())?;

        Ok(Expression::Binary {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        })
    }

    fn unary(&mut self) -> Result<Expression, String> {
        let operator = self.previous().clone();
        let right = self.parse_precedence(Precedence::Unary)?;

        Ok(Expression::Unary {
            right: Box::new(right),
            operator,
        })
    }

    fn grouping(&mut self) -> Result<Expression, String> {
        let expression = self.expression()?;
        self.chomp(Token::RightParen, "Expected ')' after expression.")?;

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
        self.tokens.get(self.current - 1).expect("some token")
    }

    fn chomp(&mut self, ref token: Token, message: &str) -> Result<(), String> {
        if self.current() == Some(token) {
            self.advance();
            Ok(())
        } else {
            Err(message.to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Expression, token::Token};

    use super::Compiler;

    #[test]
    fn single_literal() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Boolean(true)]);
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn single_variable() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Identifier(String::from("test"))]);
        let expected = Expression::Variable(Token::Identifier(String::from("test")));

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn expression_group() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![
            Token::LeftParen,
            Token::Boolean(true),
            Token::RightParen,
        ]);
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn unary_literal() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Minus, Token::Number(42.0)]);
        let expected = Expression::Unary {
            right: Box::new(Expression::Literal(Token::Number(42.0))),
            operator: Token::Minus,
        };

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn multiply_number() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Star, Token::Number(2.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(3.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Star,
        };

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn add_number() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Plus, Token::Number(2.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(3.0))),
            right: Box::new(Expression::Literal(Token::Number(2.0))),
            operator: Token::Plus,
        };

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn precedence_multiply_addition() -> Result<(), String> {
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

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn comparison_equal() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Number(5.0), Token::Equal, Token::Number(7.0)]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal(Token::Number(5.0))),
            right: Box::new(Expression::Literal(Token::Number(7.0))),
            operator: Token::Equal,
        };

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn boolean_and() -> Result<(), String> {
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

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn variable_add() -> Result<(), String> {
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

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn variable_mul() -> Result<(), String> {
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

        assert_eq!(ast, expected);
        Ok(())
    }
}
