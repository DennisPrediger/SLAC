#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Boolean(bool),
    String(String),
    Number(f64),
}
