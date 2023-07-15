use std::vec;

use crate::error::{Result, SyntaxError};
use crate::token::Token;

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
}

impl<'a> Scanner<'a> {
    pub fn tokenize(source: &'a str) -> Result<Vec<Token>> {
        let mut scanner = Scanner {
            source,
            start: 0,
            current: 0,
        };

        let mut tokens: Vec<Token> = vec![];

        while !scanner.is_at_end() {
            tokens.push(scanner.next_token()?);
        }

        if tokens.is_empty() {
            Err(SyntaxError::from("empty String"))
        } else {
            Ok(tokens)
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start = self.current;
        let next = self.next_char().unwrap();

        if Scanner::is_identifier_start(next) {
            return self.identifier();
        }

        if char::is_numeric(next) {
            return self.number();
        }

        match next {
            '\'' => self.string(),
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            ',' => Ok(Token::Comma),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '=' => Ok(Token::Equal),
            '>' => Ok(self.greater()),
            '<' => Ok(self.lesser()),
            '.' => self.number(), // interprete .1 as 0.1
            _ => Err(SyntaxError(format!("invalid token: {}", next))),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn advance_numeric(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_numeric() {
                self.advance()
            } else {
                break;
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.advance();
        self.source.chars().nth(self.current - 1)
    }

    fn peek(&self) -> Option<char> {
        self.peek_ahead(0)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.chars().nth(self.current + offset)
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ') | Some('\r') | Some('\t') | Some('\n') = self.peek() {
            self.advance();
        }
    }

    fn get_content(&self, trim_by: usize) -> Result<String> {
        let from = self.start + trim_by;
        let to = self.current - trim_by;

        let content = self
            .source
            .get(from..to)
            .ok_or(SyntaxError::from("invalid content length"))?
            .chars()
            .collect::<String>();

        Ok(content)
    }

    fn is_identifier_start(character: char) -> bool {
        character.is_alphabetic() || character == '_'
    }

    fn is_identifier(character: char) -> bool {
        character.is_alphanumeric() || character == '_' || character == '-'
    }

    fn identifier(&mut self) -> Result<Token> {
        while self.peek().is_some_and(Scanner::is_identifier) {
            self.advance();
        }

        let ident = self.get_content(0)?;

        let token = match ident.to_lowercase().as_str() {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Identifier(ident),
        };

        Ok(token)
    }

    fn extract_number(&self, content: &str) -> Result<f64> {
        content
            .parse::<f64>()
            .map_err(|o| SyntaxError(o.to_string()))
    }

    fn number(&mut self) -> Result<Token> {
        self.advance_numeric(); // advance integral

        if self.peek() == Some('.') {
            self.advance(); // advance dot

            if let Some(fractional) = self.peek() {
                if fractional.is_numeric() {
                    self.advance_numeric(); // advance fraction
                }
            }
        }

        let content = self.get_content(0)?;
        let number = self.extract_number(content.as_str())?;

        Ok(Token::Number(number))
    }

    fn string(&mut self) -> Result<Token> {
        while self.peek().is_some_and(|c| c != '\'') {
            self.advance();
        }

        if self.is_at_end() {
            let message = format!("Unterminated String at character {}", self.start);
            return Err(SyntaxError(message));
        };

        self.advance();
        let content = self.get_content(1)?;

        Ok(Token::String(content))
    }

    fn encounter_double(&mut self, token: Token) -> Token {
        self.advance();
        token
    }

    fn greater(&mut self) -> Token {
        match self.peek() {
            Some('=') => self.encounter_double(Token::GreaterEqual),
            _ => Token::Greater,
        }
    }

    fn lesser(&mut self) -> Token {
        match self.peek() {
            Some('=') => self.encounter_double(Token::LessEqual),
            Some('>') => self.encounter_double(Token::NotEqual),
            _ => Token::Less,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scanner, Token};
    use crate::error::{Result, SyntaxError};

    #[test]
    fn simple_bool() -> Result<()> {
        let tokens = Scanner::tokenize("True")?;
        let expected = Token::Boolean(true);

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_integer() -> Result<()> {
        let tokens = Scanner::tokenize("9001")?;
        let expected = Token::Number(9001.0);

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_float() -> Result<()> {
        let tokens = Scanner::tokenize("3.14")?;
        let expected = Token::Number(3.14);

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_string() -> Result<()> {
        let tokens = Scanner::tokenize("'Hello World'")?;
        let expected = Token::String(String::from("Hello World"));

        assert!(tokens.first().is_some());
        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn multiple_tokens() -> Result<()> {
        let tokens = Scanner::tokenize("1 + 1")?;
        let expected: Vec<Token> = vec![Token::Number(1.0), Token::Plus, Token::Number(1.0)];

        assert_eq!(tokens, expected);
        Ok(())
    }

    #[test]
    fn var_name_underscore() -> Result<()> {
        let tokens = Scanner::tokenize("(_SOME_VAR1 * ANOTHER-ONE)")?;
        let expected = vec![
            Token::LeftParen,
            Token::Identifier(String::from("_SOME_VAR1")),
            Token::Star,
            Token::Identifier(String::from("ANOTHER-ONE")),
            Token::RightParen,
        ];

        assert_eq!(expected, tokens);
        Ok(())
    }

    #[test]
    fn unterminated_less() -> Result<()> {
        let tokens = Scanner::tokenize("<")?;
        let expected = vec![Token::Less];

        assert_eq!(expected, tokens);
        Ok(())
    }

    fn test_number(input: &str, expected: f64) -> Result<()> {
        let tokens = Scanner::tokenize(input)?;
        let expected = vec![Token::Number(expected)];

        assert_eq!(expected, tokens);
        Ok(())
    }

    #[test]
    fn number_parts() -> Result<()> {
        test_number("10", 10.0)?;
        test_number("10.0", 10.0)?;
        test_number("20.4", 20.4)?;
        test_number("30.", 30.0)?;
        test_number(".4", 0.4)?;

        Ok(())
    }

    #[test]
    fn err_empty_input() {
        let tokens = Scanner::tokenize("");
        let expected = Err(SyntaxError::from("empty String"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unknown_token_1() {
        let tokens = Scanner::tokenize("$");
        let expected = Err(SyntaxError::from("invalid token: $"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unknown_token_2() {
        let tokens = Scanner::tokenize("$hello");
        let expected = Err(SyntaxError::from("invalid token: $"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unterminated_string() {
        let tokens = Scanner::tokenize("'hello' + 'world");
        let expected = Err(SyntaxError::from("Unterminated String at character 10"));

        assert_eq!(expected, tokens);
    }
}
