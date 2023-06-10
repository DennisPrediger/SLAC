#[rustfmt::skip]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
  // Single-character tokens
  LeftParen, RightParen,
  Plus, Minus, Star, Slash, 
  // One or two character tokens
  Greater, GreaterEqual,
  Less, LessEqual,
  // Equality
  Equal, NotEqual,
  // Keywords
  And, Or, Not,
  // Literals
  Boolean(bool),
  String(String), Number(f64),
  Identifier(String)
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Or,         // or
    And,        // and
    Equality,   // = <>
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Primary,    // Literals
}

impl From<&Token> for Precedence {
    fn from(token: &Token) -> Self {
        match token {
            Token::Minus | Token::Plus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Greater | Token::GreaterEqual => Precedence::Comparison,
            Token::Less | Token::LessEqual => Precedence::Comparison,
            Token::And => Precedence::And,
            Token::Or => Precedence::Or,
            _ => Precedence::None,
        }
    }
}

impl Precedence {
    pub fn next(self) -> Precedence {
        match self {
            Precedence::None => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}
