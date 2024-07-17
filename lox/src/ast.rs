use crate::span::Span;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum UnaryKind {
    Menos,
    Bang,
}

#[derive(Debug)]
pub enum LiteralItem {
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug)]
pub struct Literal {
    pub span: Span,
    pub item: LiteralItem,
}

#[derive(Debug)]
pub struct Expression {
    pub span: Span,
    pub item: ExpressionItem,
}

#[derive(Debug)]
pub struct Unary {
    pub span: Span,
    pub kind: UnaryKind,
    pub item: Box<Expression>,
}

#[derive(Debug)]
pub struct Binary {
    pub span: Span,
    pub items: (Box<Expression>, Box<Expression>),
    pub kind: BinaryKind,
}

#[derive(Debug)]
pub enum ExpressionItem {
    Binary(Binary),
    Unary(Unary),
    Literal(Literal),
    Grouping(Box<Expression>),
}
