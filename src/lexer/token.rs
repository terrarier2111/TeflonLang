use crate::diagnostics::span::{FixedTokenSpan, Span};
use crate::parser::keyword::Keyword;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenType {
    Ident,
    Keyword,
    BinOp,
    StrLit,
    NumLit,
    Comma,         // ,
    OpenParen,     // (
    ClosedParen,   // )
    OpenCurly,     // {
    ClosedCurly,   // }
    OpenBracket,   // [
    ClosedBracket, // ]
    Colon,         // :
    Semi,          // ;
    Apostrophe,    // '
    OpenAngle,     // <
    ClosedAngle,   // >
    Hashtag,       // #
    Star,          // *
    Dot,           // .
    Question,      // ?
    Underscore,    // _
    Arrow,         // ->
    And,           // &
    Or,            // |
    Comment,
    EOF, // end of file
    Invalid,
}

impl TokenType {
    pub fn is_buffered_token(&self) -> bool {
        // FIXME: do we even need this method?
        matches!(
            self,
            TokenType::NumLit | TokenType::StrLit | TokenType::Ident | TokenType::Keyword
        )
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Ident(Span, String),
    Keyword(Span, Keyword),
    BinOp(Span, BinOp),
    StrLit(Span, String),
    NumLit(Span, String),
    Comma(FixedTokenSpan),         // ,
    OpenParen(FixedTokenSpan),     // (
    ClosedParen(FixedTokenSpan),   // )
    OpenCurly(FixedTokenSpan),     // {
    ClosedCurly(FixedTokenSpan),   // }
    OpenBracket(FixedTokenSpan),   // [
    ClosedBracket(FixedTokenSpan), // ]
    Colon(FixedTokenSpan),         // :
    Semi(FixedTokenSpan),          // ;
    Apostrophe(FixedTokenSpan),    // '
    OpenAngle(FixedTokenSpan),     // <
    ClosedAngle(FixedTokenSpan),   // >
    Hashtag(FixedTokenSpan),       // #
    Star(FixedTokenSpan),          // *
    Dot(FixedTokenSpan),           // .
    Question(FixedTokenSpan),      // ?
    Arrow(FixedTokenSpan<2>),      // ->
    And(FixedTokenSpan),           // &
    Or(FixedTokenSpan),            // |
    Comment(Span, String),
    EOF(FixedTokenSpan), // end of file
    Invalid(FixedTokenSpan, char),
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Ident(sp, _) => *sp,
            Token::Keyword(sp, _) => *sp,
            Token::BinOp(sp, _) => *sp,
            Token::StrLit(sp, _) => *sp,
            Token::NumLit(sp, _) => *sp,
            Token::Comma(sp) => Span::single_token(sp.0),
            Token::OpenParen(sp) => Span::single_token(sp.0),
            Token::ClosedParen(sp) => Span::single_token(sp.0),
            Token::OpenCurly(sp) => Span::single_token(sp.0),
            Token::ClosedCurly(sp) => Span::single_token(sp.0),
            Token::OpenBracket(sp) => Span::single_token(sp.0),
            Token::ClosedBracket(sp) => Span::single_token(sp.0),
            Token::Colon(sp) => Span::single_token(sp.0),
            Token::Semi(sp) => Span::single_token(sp.0),
            Token::Invalid(sp, _) => Span::single_token(sp.0),
            Token::Apostrophe(sp) => Span::single_token(sp.0),
            Token::OpenAngle(sp) => Span::single_token(sp.0),
            Token::ClosedAngle(sp) => Span::single_token(sp.0),
            Token::Hashtag(sp) => Span::single_token(sp.0),
            Token::Star(sp) => Span::single_token(sp.0),
            Token::Dot(sp) => Span::single_token(sp.0),
            Token::Question(sp) => Span::single_token(sp.0),
            Token::Arrow(sp) => Span::fixed_token::<2>(sp.0),
            Token::And(sp) => Span::single_token(sp.0),
            Token::Or(sp) => Span::single_token(sp.0),
            Token::Comment(sp, _) => *sp,
            Token::EOF(sp) => Span::single_token(sp.0),
        }
    }

    pub fn to_type(&self) -> TokenType {
        match self {
            Token::Ident(_, _) => TokenType::Ident,
            Token::Keyword(_, _) => TokenType::Keyword,
            Token::BinOp(_, _) => TokenType::BinOp,
            Token::StrLit(_, _) => TokenType::StrLit,
            Token::NumLit(_, _) => TokenType::NumLit,
            Token::Comma(_) => TokenType::Comma,
            Token::OpenParen(_) => TokenType::OpenParen,
            Token::ClosedParen(_) => TokenType::ClosedParen,
            Token::OpenCurly(_) => TokenType::OpenCurly,
            Token::ClosedCurly(_) => TokenType::ClosedCurly,
            Token::OpenBracket(_) => TokenType::OpenBracket,
            Token::ClosedBracket(_) => TokenType::ClosedBracket,
            Token::Colon(_) => TokenType::Colon,
            Token::Semi(_) => TokenType::Semi,
            Token::Apostrophe(_) => TokenType::Apostrophe,
            Token::OpenAngle(_) => TokenType::OpenAngle,
            Token::ClosedAngle(_) => TokenType::ClosedAngle,
            Token::Hashtag(_) => TokenType::Hashtag,
            Token::Star(_) => TokenType::Star,
            Token::Dot(_) => TokenType::Dot,
            Token::Question(_) => TokenType::Question,
            Token::Comment(_, _) => TokenType::Comment,
            Token::Arrow(_) => TokenType::Arrow,
            Token::And(_) => TokenType::And,
            Token::Or(_) => TokenType::Or,
            Token::Invalid(_, _) => TokenType::Invalid,
            Token::EOF(_) => TokenType::EOF,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    AndEq,
    OrEq,
    AndAnd,
    OrOr,
    Eq,
    // EqEq, // ==
    // NEq,  // !=
    //?ModEq, // %=
}

impl BinOp {
    pub fn precedence(&self) -> usize {
        match self {
            BinOp::Add => 5,
            BinOp::Sub => 5,
            BinOp::Mul => 10,
            BinOp::Div => 10,
            BinOp::Mod => 10,
            BinOp::AndAnd => 2,
            BinOp::OrOr => 2,
            BinOp::AddEq => 1,
            BinOp::SubEq => 1,
            BinOp::MulEq => 1,
            BinOp::DivEq => 1,
            BinOp::AndEq => 1,
            BinOp::OrEq => 1,
            BinOp::Eq => 1,
            // BinOp::EqEq => 2,
            // BinOp::NEq => 2,
        }
    }
}
