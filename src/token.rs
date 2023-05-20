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
