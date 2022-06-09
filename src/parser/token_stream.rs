use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::lexer::token::{Token, TokenType};

pub struct TokenStream {
    tokens: Vec<Token>,
    cursor: usize,
}

impl TokenStream {

    pub fn get_next(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    pub fn eat(&mut self, token_type: TokenType) -> bool {
        let next = self.get_next();
        if let Some(next) = next {
            if next.to_type() == token_type {
                self.advance();
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn advance(&mut self) {
        self.cursor += 1;
    }

    pub fn can_advance(&self) -> bool {
        self.tokens.len() > self.cursor
    }

    pub fn look_ahead<F: Fn(&Token) -> bool>(&self, dist: usize, func: F) -> bool {
        let dist = dist.max(1) - 1;
        if let Some(token) = self.tokens.get(*self.cursor + dist) {
            func(token)
        } else {
            false
        }
    }

}

pub struct UnexpectedEOI; // FIXME: do we even need this? (we probably don't)

impl Debug for UnexpectedEOI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("an unexpected EOI occurred")
    }
}

impl Display for UnexpectedEOI {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("an unexpected EOI occurred")
    }
}

impl Error for UnexpectedEOI {}
