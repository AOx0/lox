use std::ops::{Not, Range};

pub struct Scanner<'src> {
    cursor: Cursor<'src>,
}

impl<'src> Scanner<'src> {
    pub fn new(src: &'src str) -> Scanner {
        Scanner {
            cursor: Cursor::new(src),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub span: Range<usize>,
    pub kind: ErrorKind,
}

impl Error {
    fn new(kind: ErrorKind, span: Range<usize>) -> Self {
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
        let start = self.cursor.position - 1;

        let res = match c {
            '0'..='9' => return Some(self.parse_number()),
            ' ' | '\n' | '\t' | '\r' => self.parse_space(),
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,
            '!' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenType::BangEqual
                }
                _ => TokenType::Bang,
            },
            '=' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenType::EqualEqual
                }
                _ => TokenType::Equal,
            },
            '>' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenType::GreaterEqual
                }
                _ => TokenType::Greater,
            },
            '<' => match self.cursor.peek() {
                Some('=') => {
                    self.cursor.bump();
                    TokenType::LessEqual
                }
                _ => TokenType::Less,
            },
            '/' => match self.cursor.peek() {
                Some('/') => {
                    while self.cursor.peek().is_some_and(|c| c != '\n') {
                        self.cursor.bump();
                    }
                    TokenType::CommentLine
                }
                _ => TokenType::Slash,
            },
            '"' => return Some(self.parse_string()),
            _ => {
                return Some(Err(Error::new(
                    ErrorKind::UnknownToken,
                    start..self.cursor.position,
                )))
            }
        };

        Some(Ok(Token::new(res, start..self.cursor.position)))
    }
}

impl<'src> Scanner<'src> {
    fn parse_space(&mut self) -> TokenType {
        let empty = [' ', '\t', '\r', '\n'];
        while let Some(c) = self.cursor.peek() {
            if empty.contains(&c) {
                self.cursor.bump();
            } else {
                break;
            }
        }

        TokenType::Whitespace
    }
    fn parse_number(&mut self) -> Result<Token, Error> {
        let start = self.cursor.position - 1;
        let mut punto = false;
        let mut error = false;

        loop {
            match self.cursor.peek() {
                Some('0'..='9') => self.cursor.bump(),
                Some('.') => {
                    if !punto && matches!(self.cursor.peek_nth(1), Some('0'..='9')) {
                        self.cursor.bump();
                        punto = true
                    } else if punto && matches!(self.cursor.peek_nth(1), Some('0'..='9')) {
                        self.cursor.bump();
                        error = true;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }

        if !error {
            Ok(Token::new(TokenType::Number, start..self.cursor.position))
        } else {
            Err(Error::new(
                ErrorKind::InvalidNumber,
                start..self.cursor.position,
            ))
        }
    }
    fn parse_string(&mut self) -> Result<Token, Error> {
        let start = self.cursor.position - 1;

        loop {
            match self.cursor.peek() {
                Some('"') => {
                    self.cursor.bump();
                    break;
                }
                None => {
                    return Err(Error::new(
                        ErrorKind::UnfinishedStr,
                        start..self.cursor.position,
                    ));
                }
                Some(_) => {
                    self.cursor.bump();
                }
            }; // match
        } // loop

        Ok(Token::new(TokenType::String, start..self.cursor.position))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TokenType {
    And,
    Bang,
    BangEqual,
    Class,
    Comma,
    CommentLine,
    Dot,
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

#[derive(Debug)]
pub struct Token {
    pub tipo: TokenType,
    pub span: Range<usize>,
}

impl Token {
    fn new(vtipo: TokenType, span: Range<usize>) -> Self {
        Token { tipo: vtipo, span }
    }
}
struct Cursor<'src> {
    source: &'src str,
    prev: Option<char>,
    curr: Option<char>,
    position: usize,
}
impl<'src> Cursor<'src> {
    fn new(src: &'src str) -> Cursor {
        Cursor {
            source: src,
            prev: None,
            curr: None,
            position: 0,
        }
    }

    fn prev(&self) -> Option<char> {
        self.prev
    }

    fn curr(&self) -> Option<char> {
        self.curr
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
