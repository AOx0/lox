use std::path::Path;

use crate::{ast, diag::Diagnostic, scanner::Tk};
pub use crate::{
    scanner::{Token, TokenKind},
    span::Span,
};

#[derive(Debug, Clone, Copy)]
pub struct Parser<'src> {
    ruta: &'src Path,
    source: &'src str,
    tokens: &'src [Token],
    prev: Token,
    cursor: usize,
}

#[derive(Debug)]
struct UnexpectedTokenKind {
    because: Option<TokenKind>,
    expected: Vec<TokenKind>,
    found: TokenKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedTokenKind(UnexpectedTokenKind),
    Eof,
}

type Result<T> = std::prelude::rust_2021::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub span: Span,
    pub kind: ErrorKind,
}

impl<'src> Parser<'src> {
    pub fn new(ruta: &'src Path, tokens: &'src [Token], source: &'src str) -> Parser<'src> {
        Parser {
            ruta,
            tokens,
            cursor: 0,
            source,
            prev: Token {
                tipo: TokenKind::Eof,
                span: Span::from(0..1),
            },
        }
    }

    fn try_parse<T>(&self, mut f: impl FnMut(&mut Self) -> Result<T>) -> Option<(T, usize)> {
        let mut p = *self;
        let res = f(&mut p);

        res.ok().map(|res| (res, p.cursor))
    }

    fn err_span(&self, span: Span, kind: ErrorKind) -> Error {
        Error { span, kind }
    }

    fn err(&self, kind: ErrorKind) -> Error {
        self.err_span(self.span(), kind)
    }

    fn primary(&mut self) -> Result<ast::Expression> {
        if let Some(t @ Token { tipo, span }) = self.advance() {
            match tipo {
                Tk::Number => {
                    let num = self.source[span.range()]
                        .parse()
                        .expect("The lexer does return a valid number span");
                    return Ok(ast::Expression {
                        span,
                        item: ast::ExpressionItem::Number(num),
                    });
                }
                Tk::True => {
                    return Ok(ast::Expression {
                        span,
                        item: ast::ExpressionItem::Bool(true),
                    });
                }
                Tk::False => {
                    return Ok(ast::Expression {
                        span,
                        item: ast::ExpressionItem::Bool(false),
                    });
                }
                Tk::String => {
                    return Ok(ast::Expression {
                        span,
                        item: ast::ExpressionItem::String(
                            self.source[span.range()].trim_matches('"').to_string(),
                        ),
                    });
                }
                Tk::Nil => {
                    return Ok(ast::Expression {
                        span,
                        item: ast::ExpressionItem::Nil,
                    });
                }
                TokenKind::LeftParen => {
                    let expr = self.comparison()?;

                    let token = self.peek().unwrap_or(t);
                    if token.tipo != Tk::RightParen {
                        Diagnostic::new(
                            self.source,
                            self.ruta,
                            token.span,
                            "Unclosed (".to_string(),
                        )
                        .err();
                    }

                    return Ok(expr);
                }
                x => {
                    return Err(Error {
                        span,
                        kind: ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                            because: None,
                            expected: vec![Tk::Number, Tk::True, Tk::False, Tk::String, Tk::Nil],
                            found: x,
                        }),
                    });
                }
            }
        }

        Err(Error {
            span: self.prev.span,
            kind: ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                because: None,
                expected: vec![
                    Tk::Number,
                    Tk::True,
                    Tk::False,
                    Tk::String,
                    Tk::Nil,
                    Tk::LeftParen,
                ],
                found: TokenKind::Eof,
            }),
        })
    }

    fn unary(&mut self) -> Result<ast::Expression> {
        'l: loop {
            match self.partial_next_chunk::<2>().map(|t| t.tipo) {
                [Tk::Bang, Tk::Bang] | [Tk::Minus, Tk::Minus] => {
                    self.bump_n(2);
                }
                _ => break 'l,
            }
        }

        if let Some(Token { tipo, .. }) = self.peek()
            && (tipo == Tk::Minus || tipo == Tk::Bang)
        {
            let kind = match tipo {
                Tk::Minus => ast::UnaryKind::Minus,
                Tk::Bang => ast::UnaryKind::Bang,
                _ => unreachable!("We did check it before"),
            };

            self.bump();
            let unary = match self.unary() {
                Ok(unary) => unary,
                Err(err) => {
                    Diagnostic::new(
                        self.source,
                        self.ruta,
                        err.span,
                        format!("Expected unary, but found error {err:?}"),
                    )
                    .err();
                    return self.primary();
                }
            };
            return Ok(ast::Expression {
                span: unary.span,
                item: ast::ExpressionItem::Unary(Box::new(unary), kind),
            });
        };

        self.primary()
    }

    fn factor(&mut self) -> Result<ast::Expression> {
        let mut lhs = self.unary()?;

        while let Some(Token { tipo, .. }) = self.peek()
            && (tipo == Tk::Star || tipo == Tk::Slash)
        {
            let kind = match tipo {
                Tk::Star => ast::BinaryKind::Star,
                Tk::Slash => ast::BinaryKind::Slash,
                _ => unreachable!("We did check it before"),
            };

            self.bump();
            let rhs = match self.unary() {
                Ok(rhs) => rhs,
                Err(err) => {
                    Diagnostic::new(
                        self.source,
                        self.ruta,
                        err.span,
                        format!("Expected unary, but found error {err:?}"),
                    )
                    .err();
                    break;
                }
            };

            lhs = ast::Expression {
                span: lhs.span.join(rhs.span),
                item: ast::ExpressionItem::Binary(Box::new(lhs), Box::new(rhs), kind),
            };
        }

        Ok(lhs)
    }

    fn term(&mut self) -> Result<ast::Expression> {
        let mut lhs = self.factor()?;

        while let Some(Token { tipo, .. }) = self.peek()
            && (tipo == Tk::Plus || tipo == Tk::Minus)
        {
            let kind = match tipo {
                Tk::Minus => ast::BinaryKind::Minus,
                Tk::Plus => ast::BinaryKind::Plus,
                _ => unreachable!("We did check it before"),
            };

            self.bump();
            let rhs = match self.factor() {
                Ok(rhs) => rhs,
                Err(err) => {
                    Diagnostic::new(
                        self.source,
                        self.ruta,
                        err.span,
                        format!("Expected factor, but found error {err:?}"),
                    )
                    .err();
                    break;
                }
            };

            lhs = ast::Expression {
                span: lhs.span.join(rhs.span),
                item: ast::ExpressionItem::Binary(Box::new(lhs), Box::new(rhs), kind),
            };
        }

        Ok(lhs)
    }

    fn comparison(&mut self) -> Result<ast::Expression> {
        let mut lhs = self.term()?;

        while let Some(Token { tipo, .. }) = self.peek()
            && (tipo == Tk::Less
                || tipo == Tk::LessEqual
                || tipo == Tk::GreaterEqual
                || tipo == Tk::Greater)
        {
            let kind = match tipo {
                Tk::Less => ast::BinaryKind::Less,
                Tk::LessEqual => ast::BinaryKind::LessEqual,
                Tk::GreaterEqual => ast::BinaryKind::GreaterEqual,
                Tk::Greater => ast::BinaryKind::Greater,
                _ => unreachable!("We did check it before"),
            };

            self.bump();
            let rhs = match self.term() {
                Ok(rhs) => rhs,
                Err(err) => {
                    Diagnostic::new(
                        self.source,
                        self.ruta,
                        err.span,
                        format!("Expected term, but found error {err:?}"),
                    )
                    .err();
                    break;
                }
            };

            lhs = ast::Expression {
                span: lhs.span.join(rhs.span),
                item: ast::ExpressionItem::Binary(Box::new(lhs), Box::new(rhs), kind),
            };
        }

        Ok(lhs)
    }

    fn equality(&mut self) -> Result<ast::Expression> {
        let mut lhs = self.comparison()?;

        while let Some(Token { tipo, .. }) = self.peek()
            && (tipo == Tk::EqualEqual || tipo == Tk::BangEqual)
        {
            let kind = match tipo {
                Tk::BangEqual => ast::BinaryKind::BangEqual,
                Tk::EqualEqual => ast::BinaryKind::EqualEqual,
                _ => unreachable!("We did check it before"),
            };

            self.bump();
            let rhs = match self.comparison() {
                Ok(rhs) => rhs,
                Err(err) => {
                    Diagnostic::new(
                        self.source,
                        self.ruta,
                        err.span,
                        format!("Expected comparison, but found error {err:?}"),
                    )
                    .err();
                    break;
                }
            };

            lhs = ast::Expression {
                span: lhs.span.join(rhs.span),
                item: ast::ExpressionItem::Binary(Box::new(lhs), Box::new(rhs), kind),
            };
        }

        Ok(lhs)
    }

    pub fn parse(&mut self) -> Result<ast::Expression> {
        self.equality()
        // if let Some((res, c)) = self.try_parse(Self::parse_annotated_number) {
        //     self.bump_to(c);
        //     Ok(res)
        // } else {
        //     Err(Error::Eof)
        // }
    }
}

impl Parser<'_> {
    fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn bump_to(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn bump(&mut self) {
        self.prev = self.tokens[self.cursor];
        self.cursor += 1;
    }

    fn track_bump(&mut self, track: &mut Span) {
        if let Some(t) = self.peek() {
            track.end = t.span.len();
        }
        self.bump();
    }

    fn prev_span(&self) -> Option<Span> {
        self.tokens.get(self.cursor - 1).map(|s| s.span)
    }

    fn span(&self) -> Span {
        self.prev.span
    }

    fn advance_n<const N: usize>(&mut self) -> Option<[Token; N]> {
        let tokens = *self.next_chunk::<N>()?;
        self.bump_n(N);
        Some(tokens)
    }

    fn advance(&mut self) -> Option<Token> {
        let [token] = self.advance_n::<1>()?;
        Some(token)
    }

    fn advance_track(&mut self, track: &mut Span) -> Option<Token> {
        let advance = self.advance();
        if let Some(ref t) = advance {
            track.end = t.span.end;
        }
        advance
    }

    ///
    /// ```
    /// let next3: Option<&[Token; 3]> = parser.next_chunk::<3>();
    /// ```
    fn next_chunk<const N: usize>(&self) -> Option<&[Token; N]> {
        self.tokens[self.cursor..].first_chunk::<N>()
    }

    fn partial_next_chunk<const N: usize>(&self) -> [Token; N] {
        let mut chunk = [Token::default(); N];

        let _ = (0..N).try_for_each(|i| {
            if let Some(t) = self.tokens.get(self.cursor + i).copied() {
                chunk[i] = t;
                Ok(())
            } else {
                Err(())
            }
        });

        chunk
    }

    fn lookup_n(&self, n: usize) -> Option<Token> {
        self.tokens.get(self.cursor + n - 1).copied()
    }

    fn peek(&self) -> Option<Token> {
        self.lookup_n(1)
    }
}

// #[cfg(test)]
// mod test {
//     use crate::{ast::Expression, scanner, span::Span};

//     use super::Parser;

//     #[test]
//     fn parse_expr_number() {
//         let source = "4";
//         let lexer = scanner::Scanner::new(source);
//         let tokens: Vec<_> = lexer
//             .into_iter()
//             .map(|a| a.expect("It's guaranteed to be valid"))
//             .collect();

//         let mut parser = Parser::new(&tokens, source);
//         let res = parser.parse();

//         println!("{:?}", res);

//         assert_eq!(
//             res,
//             Some(Expression {
//                 span: Span::from(0..1),
//                 item: crate::ast::ExpressionItem::Literal(crate::ast::Literal {
//                     span: Span::from(0..1),
//                     item: crate::ast::LiteralItem::Number(4.0)
//                 })
//             })
//         )
//     }

//     #[test]
//     fn parse_expr_parent() {
//         let source = "(4)";
//         let lexer = scanner::Scanner::new(source);
//         let tokens: Vec<_> = lexer
//             .into_iter()
//             .map(|a| a.expect("It's guaranteed to be valid"))
//             .collect();

//         let mut parser = Parser::new(tokens, source);
//         let res = parser.parse_expression();

//         println!("{:?}", res);

//         assert_eq!(
//             res,
//             Some(Expression {
//                 span: Span::from(0..0),
//                 item: crate::ast::ExpressionItem::Grouping(Box::new(Expression {
//                     span: Span::from(1..2),
//                     item: crate::ast::ExpressionItem::Literal(crate::ast::Literal {
//                         span: Span::from(1..2),
//                         item: crate::ast::LiteralItem::Number(4.0)
//                     })
//                 }))
//             })
//         )
//     }

//     #[test]
//     fn parse_expr_binary() {
//         let source = "(4) + (5)";
//         let lexer = scanner::Scanner::new(source);
//         let tokens: Vec<_> = lexer
//             .into_iter()
//             .map(|a| a.expect("It's guaranteed to be valid"))
//             .collect();

//         let mut parser = Parser::new(tokens, source);
//         let res = parser.parse_expression();

//         println!("{:?}", res);

//         // assert_eq!(
//         //     res,
//         //     Some(Expression {
//         //         span: Span::from(0..0),
//         //         item: crate::ast::ExpressionItem::Grouping(Box::new(Expression {
//         //             span: Span::from(1..2),
//         //             item: crate::ast::ExpressionItem::Literal(crate::ast::Literal {
//         //                 span: Span::from(1..2),
//         //                 item: crate::ast::LiteralItem::Number(4.0)
//         //             })
//         //         }))
//         //     })
//         // )
//     }
// }
