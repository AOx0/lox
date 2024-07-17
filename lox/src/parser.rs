use crate::scanner::Token;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, cursor: 0 }
    }

    fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn bump(&mut self) {
        self.cursor += 1;
    }

    ///
    /// ```
    /// let next3: Option<&[Token; 3]> = parser.next_chunk::<3>();
    /// ```
    fn next_chunk<const N: usize>(&self) -> Option<&[Token; N]> {
        self.tokens[self.cursor..].first_chunk::<N>()
    }

    fn lookup_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(n - 1)
    }

    fn peek(&self) -> Option<&Token> {
        self.lookup_n(1)
    }
}
