use crate::diagnostics::span::{SingleTokenSpan, Span};
use crate::parser::keyword::Keyword;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
    Ident,
    Keyword,
    Operator,
    StrLit,
    NumLit,
    Comma, // ,
    OpenParen, // (
    ClosedParen, // )
    OpenCurly, // {
    ClosedCurly, // }
    OpenBracket, // [
    ClosedBracket, // ]
    Eq, // =
    Colon, // :
    Semi, // ;
    Apostrophe, // "
    OpenTriangle, // <
    ClosedTriangle, // >
    Hashtag, // #
    Star, // *
    Comment,
    EOF, // end of file
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
    EOF(SingleTokenSpan), // end of file
    Invalid(SingleTokenSpan, char),
}

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
            Token::EOF(sp) => Span::single_token(sp.0),
        }
    }

    pub fn to_type(&self) -> TokenType {
        match self {
            Token::Ident(_, _) => TokenType::Ident,
            Token::Keyword(_, _) => TokenType::Keyword,
            Token::Operator(_, _) => TokenType::Operator,
            Token::StrLit(_, _) => TokenType::StrLit,
            Token::NumLit(_, _) => TokenType::NumLit,
            Token::Comma(_) => TokenType::Comma,
            Token::OpenParen(_) => TokenType::OpenParen,
            Token::ClosedParen(_) => TokenType::ClosedParen,
            Token::OpenCurly(_) => TokenType::OpenCurly,
            Token::ClosedCurly(_) => TokenType::ClosedCurly,
            Token::OpenBracket(_) => TokenType::OpenBracket,
            Token::ClosedBracket(_) => TokenType::ClosedBracket,
            Token::Eq(_) => TokenType::Eq,
            Token::Colon(_) => TokenType::Colon,
            Token::Semi(_) => TokenType::Semi,
            Token::Apostrophe(_) => TokenType::Apostrophe,
            Token::OpenTriangle(_) => TokenType::OpenTriangle,
            Token::ClosedTriangle(_) => TokenType::ClosedTriangle,
            Token::Hashtag(_) => TokenType::Hashtag,
            Token::Star(_) => TokenType::Star,
            Token::Comment(_, _) => TokenType::Comment,
            Token::Invalid(_, _) => TokenType::Invalid,
            Token::EOF(_) => TokenType::EOF,
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
