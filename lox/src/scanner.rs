use crate::span::Span;
use std::ops::Not;

type Tk = TokenKind;

pub struct Scanner<'src> {
    cursor: Cursor<'src>,
    start: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(src: &'src str) -> Scanner {
        Scanner {
            cursor: Cursor::new(src),
            start: 0,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

impl Error {
    fn new(kind: ErrorKind, span: Span) -> Self {
        Error { span, kind }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    UnfinishedStr,
    UnknownToken,
    InvalidNumber,
}

impl Iterator for Scanner<'_> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.cursor.next()?;
        self.start = self.cursor.position - 1;

        match self.parse_next(c) {
            Ok(tt) => Some(Ok(Token::new(
                tt,
                Span::from(self.start..self.cursor.position),
            ))),
            Err(err) => Some(Err(Error::new(
                err,
                Span::from(self.start..self.cursor.position),
            ))),
        }
    }
}

impl<'src> Scanner<'src> {
    fn parse_next(&mut self, c: char) -> Result<TokenKind, ErrorKind> {
        Ok(match c {
            'a'..='z' | 'A'..='Z' | '_' => self.parse_reserved().unwrap_or(Tk::Identifier),
            '0'..='9' => self.parse_number().ok_or(ErrorKind::InvalidNumber)?,
            ' ' | '\n' | '\t' | '\r' => self.parse_space(),
            '(' => Tk::LeftParen,
            ')' => Tk::RightParen,
            '{' => Tk::LeftBrace,
            '}' => Tk::RightBrace,
            ',' => Tk::Comma,
            '.' => Tk::Dot,
            '-' => Tk::Minus,
            '+' => Tk::Plus,
            ';' => Tk::Semicolon,
            '*' => Tk::Star,
            '!' => self.on_match('=', |_| Tk::BangEqual).unwrap_or(Tk::Bang),
            '=' => self.on_match('=', |_| Tk::EqualEqual).unwrap_or(Tk::Equal),
            '>' => self
                .on_match('=', |_| Tk::GreaterEqual)
                .unwrap_or(Tk::Greater),
            '<' => self.on_match('=', |_| Tk::LessEqual).unwrap_or(Tk::Less),
            '/' => self
                .on_match('/', |s| {
                    while s.cursor.peek().unwrap_or('\n') != '\n' {
                        s.cursor.bump()
                    }

                    Tk::CommentLine
                })
                .unwrap_or(Tk::Slash),
            '"' => self.parse_string().ok_or(ErrorKind::UnfinishedStr)?,
            _ => return Err(ErrorKind::UnknownToken),
        })
    }
}

impl<'src> Scanner<'src> {
    fn on_match(
        &mut self,
        char: char,
        mut action: impl FnMut(&mut Self) -> TokenKind,
    ) -> Option<TokenKind> {
        matches!(self.cursor.peek(), Some(c) if c == char).then(|| {
            self.cursor.bump();
            action(self)
        })
    }
}

impl<'src> Scanner<'src> {
    fn parse_space(&mut self) -> TokenKind {
        let empty = [' ', '\t', '\r', '\n'];
        while let Some(c) = self.cursor.peek() {
            if empty.contains(&c) {
                self.cursor.bump();
            } else {
                break;
            }
        }

        TokenKind::Whitespace
    }

    fn bump_while(&mut self, predicate: impl Fn(char) -> bool) {
        while predicate(self.cursor.peek().unwrap_or_default()) {
            self.cursor.bump()
        }
    }

    fn parse_reserved(&mut self) -> Option<TokenKind> {
        self.bump_while(|c| c.is_ascii_digit() || c.is_ascii_alphabetic() || c == '_');
        Some(match &self.cursor.orig[self.start..self.cursor.position] {
            "if" => Tk::If,
            "or" => Tk::Or,
            "and" => Tk::And,
            "for" => Tk::For,
            "fun" => Tk::Fun,
            "var" => Tk::Var,
            "nil" => Tk::Nil,
            "else" => Tk::Else,
            "true" => Tk::True,
            "this" => Tk::This,
            "class" => Tk::Class,
            "false" => Tk::False,
            "print" => Tk::Print,
            "super" => Tk::Super,
            "while" => Tk::While,
            "return" => Tk::Return,
            _ => return None,
        })
    }

    fn parse_number(&mut self) -> Option<TokenKind> {
        let mut punto = false;

        while let Some(c) = self.cursor.peek() {
            let nxt_is_num = || matches!(self.cursor.peek_nth(1), Some('0'..='9'));
            match c {
                '0'..='9' => self.cursor.bump(),
                '.' if nxt_is_num() && punto => {
                    self.bump_while(|c| c.is_ascii_digit() || c == '.');
                    return None;
                }
                '.' if nxt_is_num() && !punto => {
                    self.cursor.bump();
                    punto = true
                }
                _ => break,
            }
        }

        Some(TokenKind::Number)
    }

    fn parse_string(&mut self) -> Option<TokenKind> {
        while let Some(c) = self.cursor.peek() {
            if c == '"' {
                self.cursor.bump();
                return Some(TokenKind::String);
            } else if ['\n', '\r'].contains(&c) {
                return None;
            } else {
                self.cursor.bump();
            }
        }

        None
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub enum TokenKind {
    And,
    Bang,
    BangEqual,
    Class,
    Comma,
    CommentLine,
    Dot,
    #[default]
    Eof,
    Else,
    Equal,
    EqualEqual,
    False,
    For,
    Fun,
    Greater,
    GreaterEqual,
    If,
    Identifier,
    LeftBrace,
    LeftParen,
    Less,
    LessEqual,
    Minus,
    Nil,
    Number,
    Or,
    Print,
    Plus,
    Return,
    RightBrace,
    RightParen,
    Super,
    Semicolon,
    Slash,
    Star,
    String,
    This,
    True,
    Var,
    While,
    Whitespace,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub tipo: TokenKind,
    pub span: Span,
}

impl Token {
    fn new(vtipo: TokenKind, span: Span) -> Self {
        Token { tipo: vtipo, span }
    }
}
struct Cursor<'src> {
    source: &'src str,
    orig: &'src str,
    prev: Option<char>,
    curr: Option<char>,
    position: usize,
}
impl<'src> Cursor<'src> {
    fn new(src: &'src str) -> Cursor {
        Cursor {
            source: src,
            orig: src,
            prev: None,
            curr: None,
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.peek_nth(0)
    }

    fn peek_nth(&self, nth: usize) -> Option<char> {
        self.source.chars().nth(nth)
    }

    fn bump(&mut self) {
        if self.source.is_empty().not() {
            self.prev = self.curr;
            self.curr = self.source.chars().next();
            self.source = &self.source[1..];
            self.position += 1;
        }
    }

    fn next(&mut self) -> Option<char> {
        self.prev = self.curr;
        match self.source.chars().next() {
            Some(c) => {
                self.curr = Some(c);
                self.source = &self.source[1..];
                self.position += 1;
                Some(c)
            }
            None => None,
        }
    }
}
