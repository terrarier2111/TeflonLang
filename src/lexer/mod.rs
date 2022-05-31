use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::{SingleTokenSpan, Span};
use crate::parser::keyword::Keyword;
use crate::parser::token::{Token, TokenType};

pub fn lex(input: String) -> Result<Vec<Token>, DiagnosticBuilder> {
    let mut diagnostics_builder = DiagnosticBuilder::new(); // FIXME: add multiline support!
    let mut tokens = vec![];

    // let mut last = None;
    let mut buffer_type = BufferType::None;
    let mut buffer: Option<Vec<char>> = None;
    for x in input.chars().enumerate() {
        let mut curr_token = None;
        match x.1 {
            ':' => curr_token = Some(Token::Colon(SingleTokenSpan::new(x.0))),
            ';' => curr_token = Some(Token::Semi(SingleTokenSpan::new(x.0))),
            ',' => curr_token = Some(Token::Comma(SingleTokenSpan::new(x.0))),
            ' ' => {
                if buffer.is_some() {
                    match buffer_type {
                        BufferType::NumLit => {
                            let mut str = String::new();
                            for x in &buffer.unwrap() {
                                str.push(*x);
                            }
                            curr_token = Some(Token::NumLit(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), str));
                            buffer.unwrap().clear();
                        }
                        BufferType::StrLit => {
                            buffer.unwrap().push(' ');
                        },
                        BufferType::Other => {
                            let mut str = String::new();
                            for x in &buffer.unwrap() {
                                str.push(*x);
                            }
                            let kw = Keyword::from_str(str.as_str());
                            if let Some(kw) = kw {
                                curr_token = Some(Token::Keyword(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), kw));
                            } else {
                                curr_token = Some(Token::Ident(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), str));
                            }
                            buffer.unwrap().clear();
                        },
                        BufferType::None => {}
                    }
                }
            },
            '"' => {
                match buffer_type {
                    BufferType::NumLit => {
                        let mut str = String::new();
                        for x in &buffer.unwrap() {
                            str.push(*x);
                        }
                        tokens.push(Token::NumLit(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), str));
                        buffer_type = BufferType::StrLit;
                        buffer.unwrap().clear();
                    },
                    BufferType::StrLit => {
                        let mut str = String::new();
                        for x in &buffer.unwrap() {
                            str.push(*x);
                        }
                        tokens.push(Token::StrLit(Span::multi_token(x.0 - 1 - 1 - buffer.unwrap().len(), x.0 + 1), str));
                        buffer_type = BufferType::None;
                        buffer.unwrap().clear();
                    },
                    BufferType::Other => {
                        let mut str = String::new();
                        for x in &buffer.unwrap() {
                            str.push(*x);
                        }
                        let kw = Keyword::from_str(str.as_str());
                        if let Some(kw) = kw {
                            tokens.push(Token::Keyword(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), kw));
                        } else {
                            tokens.push(Token::Ident(Span::multi_token(x.0 - 1 - buffer.unwrap().len(), x.0), str));
                        }
                        buffer_type = BufferType::StrLit;
                        buffer.unwrap().clear();
                    },
                    BufferType::None => {
                        buffer = Some(vec![]);
                        buffer_type = BufferType::StrLit;
                    },
                }
                curr_token = Some(Token::Apostrophe(SingleTokenSpan::new(x.0)));
            },
            '(' => curr_token = Some(Token::OpenParen(SingleTokenSpan::new(x.0))),
            ')' => curr_token = Some(Token::ClosedParen(SingleTokenSpan::new(x.0))),
            '0'..='9' => {
                if buffer.is_none() {
                    buffer = Some(vec![]);
                    if buffer_type == BufferType::None {
                        buffer_type = BufferType::NumLit;
                    }
                }
                buffer.unwrap().push(x.1); // FIXME: we could use unwrap_unchecked
            },
            (('a'..='z') | ('A'..='Z') | '_') => {
                if buffer.is_none() {
                    buffer = Some(vec![]);
                    if buffer_type == BufferType::None {
                        buffer_type = BufferType::Other;
                    }
                } else if buffer_type == BufferType::NumLit {
                    // FIXME: error, cuz we don't expect non-number chars in NumLits
                }
                buffer.unwrap().push(x.1); // FIXME: we could use unwrap_unchecked
            },
            '{' => curr_token = Some(Token::OpenCurly(SingleTokenSpan::new(x.0))),
            '}' => curr_token = Some(Token::ClosedCurly(SingleTokenSpan::new(x.0))),
            '[' => curr_token = Some(Token::OpenBracket(SingleTokenSpan::new(x.0))),
            ']' => curr_token = Some(Token::ClosedBracket(SingleTokenSpan::new(x.0))),
            '<' => curr_token = Some(Token::OpenTriangle(SingleTokenSpan::new(x.0))),
            '>' => curr_token = Some(Token::ClosedTriangle(SingleTokenSpan::new(x.0))),
            '#' => curr_token = Some(Token::Hashtag(SingleTokenSpan::new(x.0))),
            _ => curr_token = Some(Token::Invalid(SingleTokenSpan::new(x.0), x.1)),
        }
        if let Some(token) = curr_token.take() {
            tokens.push(token);
        }
    }

    if diagnostics_builder.is_empty() {
       Ok(tokens)
    } else {
        Err(diagnostics_builder)
    }
}

#[derive(PartialEq, Copy, Clone)]
enum BufferType {
    NumLit,
    StrLit,
    Other,
    None,
}