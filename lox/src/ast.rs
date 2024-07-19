use crate::span::Span;

#[derive(Debug, PartialEq)]
pub enum BinaryKind {
    Sum,
    Sub,
    Mul,
    Div,
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
    Menos,
    Bang,
}

#[derive(Debug, PartialEq)]
pub struct Expression {
    pub span: Span,
    pub item: ExpressionItem,
}

#[derive(Debug, PartialEq)]
pub struct Unary {
    pub span: Span,
    pub kind: UnaryKind,
    pub item: Box<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Binary {
    pub span: Span,
    pub items: (Box<Expression>, Box<Expression>),
    pub kind: BinaryKind,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionItem {
    Binary(Binary),
    Unary(Unary),
    Number(f64),
    Grouping(Box<Expression>),
}
