use crate::{ast::Expression, token::Token};

pub struct Compiler {
    tokens: Vec<Token>,
    current: usize,
}

macro_rules! match_tokens {
    ($sel:ident, $( $x:expr),*) => {
        {
            if $($sel.check($x) )||* {
                $sel.advance();
                true
            } else {
                false
            }
        }
    };
}

impl Compiler {
    pub fn compile_ast(tokens: Vec<Token>) -> Result<Expression, String> {
        let mut compiler = Compiler { tokens, current: 0 };

        compiler.compile()
    }

    fn compile(&mut self) -> Result<Expression, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expression, String> {
        self.or()
    }

    fn or(&mut self) -> Result<Expression, String> {
        let mut expression = self.and()?;

        while match_tokens!(self, Token::Or) {
            let operator = self.previous().clone();
            let right = self.and()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn and(&mut self) -> Result<Expression, String> {
        let mut expression = self.equality()?;

        while match_tokens!(self, Token::And) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn equality(&mut self) -> Result<Expression, String> {
        let mut expression = self.comparison()?;

        while match_tokens!(self, Token::Equal, Token::NotEqual) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        let mut expression = self.addition()?;

        while match_tokens!(
            self,
            Token::Greater,
            Token::GreaterEqual,
            Token::Less,
            Token::LessEqual
        ) {
            let operator = self.previous().clone();
            let right = self.addition()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn addition(&mut self) -> Result<Expression, String> {
        let mut expression = self.multipication()?;

        while match_tokens!(self, Token::Plus, Token::Minus) {
            let operator = self.previous().clone();
            let right = self.multipication()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn multipication(&mut self) -> Result<Expression, String> {
        let mut expression = self.unary()?;

        while match_tokens!(self, Token::Star, Token::Slash) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                right: Box::new(right),
                operator,
            }
        }

        Ok(expression)
    }

    fn unary(&mut self) -> Result<Expression, String> {
        if match_tokens!(self, Token::Not, Token::Minus) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(Expression::Unary {
                right: Box::new(right),
                operator: operator,
            })
        } else {
            self.primary()
        }
    }

    fn grouping(&mut self) -> Result<Expression, String> {
        let expression = self.expression()?;
        self.chomp(Token::RightParen, "Expected ')' after expression.")?;

        Ok(expression)
    }

    fn primary(&mut self) -> Result<Expression, String> {
        self.advance();
        let current = self.previous();

        match current {
            Token::Boolean(_) | Token::String(_) | Token::Number(_) => {
                Ok(Expression::Literal(current.clone()))
            }
            Token::Identifier(_) => Ok(Expression::Variable(current.clone())),
            Token::LeftParen => self.grouping(),
            _ => Err("Expected literal Value".to_string()),
        }
    }

    fn check(&self, ref token: Token) -> bool {
        Some(token) == self.tokens.get(self.current)
    }

    fn advance(&mut self) {
        if self.current <= self.tokens.len() - 1 {
            self.current += 1;
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn chomp(&mut self, ref token: Token, message: &str) -> Result<(), String> {
        if self.current() == token {
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
        let ast = Compiler::compile_ast(vec![Token::Boolean(true)])?;
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn single_variable() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Identifier(String::from("test"))])?;
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
        ])?;
        let expected = Expression::Literal(Token::Boolean(true));

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn unary_literal() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Minus, Token::Number(42.0)])?;
        let expected = Expression::Unary {
            right: Box::new(Expression::Literal(Token::Number(42.0))),
            operator: Token::Minus,
        };

        assert_eq!(ast, expected);
        Ok(())
    }

    #[test]
    fn multiply_number() -> Result<(), String> {
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Star, Token::Number(2.0)])?;
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
        let ast = Compiler::compile_ast(vec![Token::Number(3.0), Token::Plus, Token::Number(2.0)])?;
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
        ])?;
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
        let ast =
            Compiler::compile_ast(vec![Token::Number(5.0), Token::Equal, Token::Number(7.0)])?;
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
        ])?;
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
        ])?;
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
        ])?;
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
