use std::ops::{Not, Range};

type Tt = TokenKind;

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

        match self.parse_next(c) {
            Ok(tt) => Some(Ok(Token::new(tt, start..self.cursor.position))),
            Err(err) => Some(Err(Error::new(err, start..self.cursor.position))),
        }
    }
}

impl<'src> Scanner<'src> {
    fn parse_next(&mut self, c: char) -> Result<TokenKind, ErrorKind> {
        Ok(match c {
            '0'..='9' => self.parse_number().ok_or(ErrorKind::InvalidNumber)?,
            ' ' | '\n' | '\t' | '\r' => self.parse_space(),
            '(' => Tt::LeftParen,
            ')' => Tt::RightParen,
            '{' => Tt::LeftBrace,
            '}' => Tt::RightBrace,
            ',' => Tt::Comma,
            '.' => Tt::Dot,
            '-' => Tt::Minus,
            '+' => Tt::Plus,
            ';' => Tt::Semicolon,
            '*' => Tt::Star,
            '!' => self.on_match('=', |_| Tt::BangEqual).unwrap_or(Tt::Bang),
            '=' => self.on_match('=', |_| Tt::EqualEqual).unwrap_or(Tt::Equal),
            '>' => self
                .on_match('=', |_| Tt::GreaterEqual)
                .unwrap_or(Tt::Greater),
            '<' => self.on_match('=', |_| Tt::LessEqual).unwrap_or(Tt::Less),
            '/' => self
                .on_match('/', |s| {
                    while s.cursor.peek().unwrap_or('\n') != '\n' {
                        s.cursor.bump()
                    }

                    Tt::CommentLine
                })
                .unwrap_or(Tt::Slash),
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
            self.cursor.bump();
            if c == '"' {
                return Some(TokenKind::String);
            }
        }

        None
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TokenKind {
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
    pub tipo: TokenKind,
    pub span: Range<usize>,
}

impl Token {
    fn new(vtipo: TokenKind, span: Range<usize>) -> Self {
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
