use crate::diagnostics::span::{SingleTokenSpan, Span};
use crate::parser::keyword::Keyword;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
    Ident,
    Keyword,
    Operator,
    StrLit,
    NumLit,
    Comma,
    OpenParen,
    ClosedParen,
    OpenCurly,
    ClosedCurly,
    OpenBracket,
    ClosedBracket,
    Eq,
    Colon,
    Semi,
    Apostrophe,
    Invalid,
}

impl TokenType {

    pub fn is_buffered_token(&self) -> bool {
         matches!(self, TokenType::NumLit | TokenType::StrLit | TokenType::Ident | TokenType::Keyword)
    }

}

#[derive(Debug)]
pub enum Token {
    Ident(Span, String),
    Keyword(Span, Keyword),
    Operator(SingleTokenSpan, Operator),
    StrLit(Span, String),
    NumLit(Span, String),
    Comma(SingleTokenSpan), // ,
    OpenParen(SingleTokenSpan), // (
    ClosedParen(SingleTokenSpan), // )
    OpenCurly(SingleTokenSpan), // {
    ClosedCurly(SingleTokenSpan), // }
    OpenBracket(SingleTokenSpan), // [
    ClosedBracket(SingleTokenSpan), // ]
    Eq(SingleTokenSpan), // =
    Colon(SingleTokenSpan), // :
    Semi(SingleTokenSpan), // ;
    Apostrophe(SingleTokenSpan), // "
    OpenTriangle(SingleTokenSpan), // <
    ClosedTriangle(SingleTokenSpan), // >
    Hashtag(SingleTokenSpan), // #
    Star(SingleTokenSpan), // *
    Comment(Span, String),
    Invalid(SingleTokenSpan, char),
}

/*
enum Bewegungsmittel {
    Flugzeug(u32, f64, u32), // turbinen, flügellänge, sitze
    Beine(u8, f64), // anzahl, länge
}

impl Bewegungsmittel {

    fn geschwindigkeit(&self) -> f64 {
        match self {
            Bewegungsmittel::Flugzeug(turb, flug, _) => (*turb) as f64 * *flug,
            Bewegungsmittel::Beine(anz, lang) => (*anz) as f64 * *lang,
        }
    }

}*/

impl Token {

    pub fn span(&self) -> Span {
        match self {
            Token::Ident(sp, _) => *sp,
            Token::Keyword(sp, _) => *sp,
            Token::Operator(sp, _) => Span::single_token(sp.0),
            Token::StrLit(sp, _) => *sp,
            Token::NumLit(sp, _) => *sp,
            Token::Comma(sp) => Span::single_token(sp.0),
            Token::OpenParen(sp) => Span::single_token(sp.0),
            Token::ClosedParen(sp) => Span::single_token(sp.0),
            Token::OpenCurly(sp) => Span::single_token(sp.0),
            Token::ClosedCurly(sp) => Span::single_token(sp.0),
            Token::OpenBracket(sp) => Span::single_token(sp.0),
            Token::ClosedBracket(sp) => Span::single_token(sp.0),
            Token::Eq(sp) => Span::single_token(sp.0),
            Token::Colon(sp) => Span::single_token(sp.0),
            Token::Semi(sp) => Span::single_token(sp.0),
            Token::Invalid(sp, _) => Span::single_token(sp.0),
            Token::Apostrophe(sp) => Span::single_token(sp.0),
            Token::OpenTriangle(sp) => Span::single_token(sp.0),
            Token::ClosedTriangle(sp) => Span::single_token(sp.0),
            Token::Hashtag(sp) => Span::single_token(sp.0),
            Token::Star(sp) => Span::single_token(sp.0),
            Token::Comment(sp, _) => *sp,
        }
    }

}

#[derive(Copy, Clone, Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
