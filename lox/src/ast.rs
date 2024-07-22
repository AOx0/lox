use crate::span::Span;

#[derive(Debug, PartialEq)]
pub enum BinaryKind {
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum UnaryKind {
    Minus,
    Bang,
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub span: Span,
    pub item: ExpressionItem,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionItem {
    Binary(Box<Expression>, Box<Expression>, BinaryKind),
    Unary(Box<Expression>, UnaryKind),
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Grouping(Box<Expression>),
}
