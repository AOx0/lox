use std::{ops::Not, path::Path};

use crate::ast;
pub use crate::{
    scanner::{Token, TokenKind},
    span::Span,
};

#[derive(Debug, Clone, Copy)]
pub struct Parser<'src> {
    ruta: &'src Path,
    source: &'src str,
    tokens: &'src [Token],
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
        }
    }

    fn try_parse<T>(&self, mut f: impl FnMut(&mut Self) -> Result<T, Error>) -> Option<(T, usize)> {
        let mut p = *self;
        let res = f(&mut p);

        res.ok().map(|res| (res, p.cursor))
    }

    fn err_span(&self, span: Span, kind: ErrorKind) -> Error {
        Error { span, kind }
    }

    fn err(&self, kind: ErrorKind) -> Error {
        self.err_span(self.span().unwrap_or(Span::from(0..1)), kind)
    }

    fn annotated_number(&mut self) -> Result<f64, Error> {
        let curr = self.advance().ok_or(self.err(ErrorKind::Eof))?;
        let mut track = curr.span;

        match curr.tipo {
            x if matches!(x, TokenKind::Minus | TokenKind::Plus) => {
                let number = self
                    .advance_track(&mut track)
                    .ok_or(self.err_span(track, ErrorKind::Eof))?;

                if !matches!(number.tipo, TokenKind::Number) {
                    return Err(self.err_span(
                        track,
                        ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                            because: Some(x),
                            found: number.tipo,
                            expected: vec![TokenKind::Number],
                        }),
                    ));
                }

                Ok(if matches!(x, TokenKind::Minus) {
                    -1.
                } else {
                    1.
                } * self.source[number.span.range()]
                    .parse::<f64>()
                    .expect("The lexer returns valid numbers"))
            }
            TokenKind::Number => Ok(self.source[curr.span.range()]
                .parse()
                .expect("The lexer returns valid numbers")),
            x => Err(self.err_span(
                track,
                ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                    because: None,
                    expected: vec![TokenKind::Number, TokenKind::Plus, TokenKind::Minus],
                    found: x,
                }),
            )),
        }
    }

    fn op_mul(&mut self) -> Result<ast::Expression, Error> {
        let mut track = self.span().unwrap_or_default();
        let rhs = if let Some((mul, c)) = self.try_parse(Self::op_mul) {
            self.bump_to(c);
            mul
        } else {
            self.num_or_group()?
        };

        let op = match self
            .advance_track(&mut track)
            .map(|t| t.tipo)
            .unwrap_or_default()
        {
            x @ TokenKind::Plus => x,
            x @ TokenKind::Minus => x,
            x => {
                return Err(self.err_span(
                    track,
                    ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                        because: None,
                        expected: vec![TokenKind::Plus, TokenKind::Minus],
                        found: x,
                    }),
                ))
            }
        };

        self.num_or_group()
    }

    fn mul_or_num(&mut self) -> Result<ast::Expression, Error> {
        if let Some((mul, c)) = self.try_parse(Self::op_mul) {
            self.bump_to(c);
            Ok(mul)
        } else {
            let start = self.span().unwrap_or_default();
            let num = self.annotated_number()?;
            let end = start.join(self.prev_span().unwrap_or_default());
            Ok(ast::Expression {
                span: end,
                item: ast::ExpressionItem::Number(num),
            })
        }
    }

    fn op_sum(&mut self) -> Result<ast::Expression, Error> {
        let mut track = self.span().unwrap_or_default();
        let expr = if let Some((sum, c)) = self.try_parse(Self::op_sum) {
            self.bump_to(c);
            sum
        } else if let Some((mul_num, c)) = self.try_parse(Self::mul_or_num) {
            self.bump_to(c);
            mul_num
        } else {
            todo!()
        };

        let op = match self
            .advance_track(&mut track)
            .map(|t| t.tipo)
            .unwrap_or_default()
        {
            x @ TokenKind::Plus => x,
            x @ TokenKind::Minus => x,
            x => {
                return Err(self.err_span(
                    track,
                    ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                        because: None,
                        expected: vec![TokenKind::Plus, TokenKind::Minus],
                        found: x,
                    }),
                ))
            }
        };

        self.mul_or_num()
    }

    fn expr(&mut self) -> Result<ast::Expression, Error> {
        if let Some((sum, c)) = self.try_parse(Self::op_sum) {
            self.bump_to(c);
            Ok(sum)
        } else if let Some((mul, c)) = self.try_parse(Self::op_mul) {
            self.bump_to(c);
            Ok(mul)
        } else if let Some((num_group, c)) = self.try_parse(Self::num_or_group) {
            self.bump_to(c);
            Ok(num_group)
        } else {
            panic!()
        }
    }

    fn num_or_group(&mut self) -> Result<ast::Expression, Error> {
        if let Some((num, c)) = self.try_parse(Self::annotated_number) {
            let start = self.span().unwrap_or_default();
            self.bump_to(c);
            let end = start.join(self.prev_span().unwrap_or_default());
            Ok(ast::Expression {
                span: end,
                item: ast::ExpressionItem::Number(num),
            })
        } else {
            self.group()
        }
    }

    fn group(&mut self) -> Result<ast::Expression, Error> {
        let token = self.advance().ok_or(self.err(ErrorKind::Eof))?;
        let mut track = token.span;

        if matches!(token.tipo, TokenKind::LeftParen).not() {
            return Err(self.err_span(
                track,
                ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                    because: None,
                    expected: vec![TokenKind::LeftParen],
                    found: token.tipo,
                }),
            ));
        }

        let expr = self.expr()?;
        track.set_end(self.prev_span().unwrap_or_default().end());
        let tipo = self
            .advance_track(&mut track)
            .map(|a| a.tipo)
            .unwrap_or(TokenKind::Eof);

        if matches!(tipo, TokenKind::RightParen).not() {
            return Err(self.err_span(
                track,
                ErrorKind::UnexpectedTokenKind(UnexpectedTokenKind {
                    because: None,
                    expected: vec![TokenKind::RightParen],
                    found: tipo,
                }),
            ));
        }

        Ok(expr)
    }

    pub fn parse(&mut self) -> Result<ast::Expression, Error> {
        self.expr()

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
        self.cursor += 1;
    }

    fn track_bump(&mut self, track: &mut Span) {
        if let Some(t) = self.peek() {
            track.set_end(t.span.len());
        }
        self.bump();
    }

    fn prev_span(&self) -> Option<Span> {
        self.tokens.get(self.cursor - 1).map(|s| s.span)
    }

    fn span(&self) -> Option<Span> {
        self.tokens.get(self.cursor).map(|s| s.span)
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
            track.set_end(t.span.end())
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
