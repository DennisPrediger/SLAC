use crate::token::Token;

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return None;
        }

        let next_char = self.next_char();

        if Scanner::is_identifier_start(next_char) {
            return self.identifier();
        }

        if next_char.is_numeric() {
            return self.number();
        }

        match next_char {
            '\'' => self.string(),
            '(' => self.single(Token::LeftParen),
            ')' => self.single(Token::RightParen),
            '+' => self.single(Token::Plus),
            '-' => self.single(Token::Minus),
            '*' => self.single(Token::Star),
            '/' => self.single(Token::Slash),
            '=' => self.single(Token::Equal),
            '>' => self.greater(),
            '<' => self.lesser(),
            _ => None,
        }
    }
}

impl<'a> Scanner<'a> {
    pub fn tokenize(source: &'a str) -> Vec<Token> {
        let scanner = Scanner {
            source,
            start: 0,
            current: 0,
        };

        scanner.collect()
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

    fn next_char(&mut self) -> char {
        self.advance();
        self.source.chars().nth(self.current - 1).unwrap()
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

    fn get_content(&self, trim_by: usize) -> String {
        self.source
            .get(self.start + trim_by..self.current - trim_by)
            .unwrap()
            .chars()
            .collect::<String>()
    }

    fn is_identifier_start(character: char) -> bool {
        character.is_alphabetic() || character == '_'
    }

    fn is_identifier(character: char) -> bool {
        character.is_alphanumeric() || character == '_' || character == '-'
    }

    fn identifier(&mut self) -> Option<Token> {
        while self.peek().is_some_and(Scanner::is_identifier) {
            self.advance();
        }

        let ident = self.get_content(0);

        match ident.to_lowercase().as_str() {
            "true" => Some(Token::Boolean(true)),
            "false" => Some(Token::Boolean(false)),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "not" => Some(Token::Not),
            _ => Some(Token::Identifier(ident)),
        }
    }

    fn number(&mut self) -> Option<Token> {
        self.advance_numeric();

        if self.peek() == Some('.') {
            match self.peek_ahead(1) {
                Some(fractional) if fractional.is_numeric() => {
                    self.advance(); // advance dot
                    self.advance_numeric(); // advance fraction
                }
                _ => (),
            }
        }

        match self.get_content(0).parse::<f64>() {
            Ok(number) => Some(Token::Number(number)),
            Err(_) => None,
        }
    }

    fn string(&mut self) -> Option<Token> {
        while self.peek().is_some_and(|c| c != '\'') {
            self.advance();
        }

        if self.is_at_end() {
            None
        } else {
            self.advance();
            Some(Token::String(self.get_content(1)))
        }
    }

    fn single(&self, token: Token) -> Option<Token> {
        Some(token)
    }

    fn greater(&mut self) -> Option<Token> {
        match self.peek() {
            Some('=') => {
                self.advance();
                Some(Token::GreaterEqual)
            }
            Some(_) => Some(Token::Greater),
            _ => None,
        }
    }

    fn lesser(&mut self) -> Option<Token> {
        match self.peek() {
            Some('=') => {
                self.advance();
                Some(Token::LessEqual)
            }
            Some('>') => {
                self.advance();
                Some(Token::NotEqual)
            }
            Some(_) => Some(Token::Less),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scanner, Token};

    #[test]
    fn simple_bool() {
        let tokens = Scanner::tokenize("True");
        let expected = Token::Boolean(true);

        assert_eq!(tokens[0], expected);
    }

    #[test]
    fn simple_integer() {
        let tokens = Scanner::tokenize("9001");
        let expected = Token::Number(9001.0);

        assert_eq!(tokens[0], expected);
    }

    #[test]
    fn simple_float() {
        let tokens = Scanner::tokenize("3.14");
        let expected = Token::Number(3.14);

        assert_eq!(tokens[0], expected);
    }

    #[test]
    fn simple_string() {
        let tokens = Scanner::tokenize("'Hello World'");
        let expected = Token::String(String::from("Hello World"));

        assert!(tokens.first().is_some());
        assert_eq!(tokens[0], expected);
    }

    #[test]
    fn multiple_tokens() {
        let tokens = Scanner::tokenize("1 + 1");
        let expected: Vec<Token> = vec![Token::Number(1.0), Token::Plus, Token::Number(1.0)];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn var_name_underscore() {
        let tokens = Scanner::tokenize("(SOME_VAR1 * ANOTHER-ONE)");
        let expected = vec![
            Token::LeftParen,
            Token::Identifier(String::from("SOME_VAR1")),
            Token::Star,
            Token::Identifier(String::from("ANOTHER-ONE")),
            Token::RightParen,
        ];

        assert_eq!(expected, tokens)
    }
}
