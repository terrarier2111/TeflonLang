use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::{FixedTokenSpan, Span};
use crate::lexer::token::{BinOp, Token};
use crate::parser::keyword::Keyword;

pub mod token;

pub fn lex(input: String) -> Result<Vec<Token>, DiagnosticBuilder> {
    let input = input.chars().collect::<Vec<char>>();
    let mut cursor = 0_usize;
    let mut diagnostics_builder = DiagnosticBuilder::new();
    let mut tokens = vec![];

    ///
    ///
    /// args:
    ///
    /// cursor: the last processed token index
    ///
    fn read_into_buffer<F: Fn(char) -> bool>(
        input: &[char],
        cursor: usize,
        do_continue: F,
    ) -> (String, usize) {
        let mut buffer = String::new();
        let mut new_cursor = cursor/* + 1*/;
        while input.len() > new_cursor && do_continue(input[new_cursor]) {
            buffer.push(input[new_cursor]);
            new_cursor += 1;
        }
        (buffer, new_cursor)
    }

    while input.len() > cursor {
        let mut curr_token = None;
        let curr = input[cursor];
        match curr {
            ' ' => {}
            '"' => {
                let (buffer, new_cursor) = read_into_buffer(&input, cursor + 1, |x| x != '"');
                curr_token = Some(Token::StrLit(Span::multi_token(cursor, new_cursor), buffer));
                cursor = new_cursor - 1;
            }
            '0'..='9' => {
                let (buffer, new_cursor) =
                    read_into_buffer(&input, cursor, |x| matches!(x, '.' | ('0'..='9')));
                curr_token = Some(Token::NumLit(Span::multi_token(cursor, new_cursor), buffer));
                cursor = new_cursor - 1;
            }
            (('a'..='z') | ('A'..='Z') | '_') => {
                let (buffer, new_cursor) = read_into_buffer(&input, cursor, |x| {
                    matches!(x, ('a'..='z') | ('A'..='Z') | ('0'..='9') | '_')
                });
                let kw = Keyword::from_str(buffer.as_str());
                if let Some(kw) = kw {
                    curr_token = Some(Token::Keyword(Span::multi_token(cursor, new_cursor), kw));
                } else {
                    curr_token = Some(Token::Ident(Span::multi_token(cursor, new_cursor), buffer));
                }
                cursor = new_cursor - 1;
            }
            '(' => curr_token = Some(Token::OpenParen(FixedTokenSpan::new(cursor))),
            ')' => curr_token = Some(Token::ClosedParen(FixedTokenSpan::new(cursor))),
            '{' => curr_token = Some(Token::OpenCurly(FixedTokenSpan::new(cursor))),
            '}' => curr_token = Some(Token::ClosedCurly(FixedTokenSpan::new(cursor))),
            '[' => curr_token = Some(Token::OpenBracket(FixedTokenSpan::new(cursor))),
            ']' => curr_token = Some(Token::ClosedBracket(FixedTokenSpan::new(cursor))),
            '<' => curr_token = Some(Token::OpenAngle(FixedTokenSpan::new(cursor))),
            '>' => curr_token = Some(Token::ClosedAngle(FixedTokenSpan::new(cursor))),
            ':' => curr_token = Some(Token::Colon(FixedTokenSpan::new(cursor))),
            ';' => curr_token = Some(Token::Semi(FixedTokenSpan::new(cursor))),
            ',' => curr_token = Some(Token::Comma(FixedTokenSpan::new(cursor))),
            '#' => curr_token = Some(Token::Hashtag(FixedTokenSpan::new(cursor))),
            '\'' => curr_token = Some(Token::Apostrophe(FixedTokenSpan::new(cursor))),
            '?' => curr_token = Some(Token::Question(FixedTokenSpan::new(cursor))),
            '_' => curr_token = Some(Token::Underscore(FixedTokenSpan::new(cursor))),
            '.' => curr_token = Some(Token::Dot(FixedTokenSpan::new(cursor))),
            '=' => curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Eq)),
            '/' => match input[cursor + 1] {
                '/' => {
                    let span_start = cursor;
                    cursor += 2;
                    let mut buffer = String::new();
                    while input.len() > cursor && input[cursor] != '\n' {
                        buffer.push(input[cursor]);
                        cursor += 1;
                    }
                    cursor -= 1;
                    curr_token = Some(Token::Comment(
                        Span::multi_token(span_start, cursor),
                        buffer,
                    ));
                }
                '=' => {
                    curr_token = Some(Token::BinOp(
                        Span::multi_token(cursor, cursor + 1),
                        BinOp::DivEq,
                    ))
                }
                _ => curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Div)),
            },
            '+' => {
                if input[cursor + 1] == '=' {
                    curr_token = Some(Token::BinOp(
                        Span::multi_token(cursor, cursor + 1),
                        BinOp::AddEq,
                    ));
                } else {
                    curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Add));
                }
            }
            '-' => {
                let token = match input[cursor + 1] {
                    '=' => {
                        let ret = Token::BinOp(Span::multi_token(cursor, cursor + 1), BinOp::SubEq);
                        cursor += 1;
                        ret
                    }
                    '>' => {
                        let ret = Token::Arrow(FixedTokenSpan::new(cursor));
                        cursor += 1;
                        ret
                    }
                    _ => Token::BinOp(Span::single_token(cursor), BinOp::Sub),
                };
                curr_token = Some(token);
            }
            '*' => {
                if input[cursor + 1] == '=' {
                    curr_token = Some(Token::BinOp(
                        Span::multi_token(cursor, cursor + 1),
                        BinOp::MulEq,
                    ));
                } else {
                    curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Mul));
                }
            }
            ('\r' | '\n') => {} // this is a noop
            _ => {
                curr_token = Some(Token::Invalid(FixedTokenSpan::new(cursor), curr));
            }
        }
        if let Some(token) = curr_token.take() {
            tokens.push(token);
        }
        cursor += 1;
    }

    tokens.push(Token::EOF(FixedTokenSpan::new(input.as_slice().len())));

    if diagnostics_builder.is_empty() {
        Ok(tokens)
    } else {
        Err(diagnostics_builder)
    }
}
