use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::{SingleTokenSpan, Span};
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
    fn read_into_buffer<F: Fn(char) -> bool>(input: &[char], cursor: usize, do_continue: F) -> (String, usize) {
        let mut buffer = String::new();
        let mut new_cursor = cursor/* + 1*/;
        while input.len() > new_cursor && do_continue(input[new_cursor]) {
            buffer.push(input[new_cursor]);
            new_cursor += 1;
        }
        (buffer, new_cursor)
    }


    /*

    ****************************************************************
                                    NEW
     */

    while input.len() > cursor {
        let mut curr_token = None;
        let curr = input[cursor];
        match curr {
            ' ' => {
                // FIXME: fix this
                /*if buffer.is_some() {
                    match buffer_type {
                        BufferType::NumLit => {
                            let mut str = String::new();
                            for x in &buffer.unwrap() {
                                str.push(*curr);
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
                }*/
            },
            '"' => {
                let (buffer, new_cursor) = read_into_buffer(&input, cursor + 1, |x| x != '"');
                curr_token = Some(Token::StrLit(Span::multi_token(cursor, new_cursor), buffer));
                cursor = new_cursor;
            },
            '0'..='9' => {
                let (buffer, new_cursor) = read_into_buffer(&input, cursor, |x| matches!(x, '.' | ('0'..='9')));
                curr_token = Some(Token::NumLit(Span::multi_token(cursor, new_cursor), buffer));
                cursor = new_cursor;
            },
            (('a'..='z') | ('A'..='Z') | '_') => {
                let (buffer, new_cursor) = read_into_buffer(&input, cursor, |x| {
                    /*match x {
                        (('a'..='z') | ('A'..='Z') | ('0'..='9') | '_') => true,
                        _ => false,
                    }*/
                    matches!(x, ('a'..='z') | ('A'..='Z') | ('0'..='9') | '_')
                });
                let kw = Keyword::from_str(buffer.as_str());
                if let Some(kw) = kw {
                    curr_token = Some(Token::Keyword(Span::multi_token(cursor, new_cursor), kw));
                } else {
                    curr_token = Some(Token::Ident(Span::multi_token(cursor, new_cursor), buffer));
                }
                cursor = new_cursor;
            },
            '(' => curr_token = Some(Token::OpenParen(SingleTokenSpan::new(cursor))),
            ')' => curr_token = Some(Token::ClosedParen(SingleTokenSpan::new(cursor))),
            '{' => curr_token = Some(Token::OpenCurly(SingleTokenSpan::new(cursor))),
            '}' => curr_token = Some(Token::ClosedCurly(SingleTokenSpan::new(cursor))),
            '[' => curr_token = Some(Token::OpenBracket(SingleTokenSpan::new(cursor))),
            ']' => curr_token = Some(Token::ClosedBracket(SingleTokenSpan::new(cursor))),
            '<' => curr_token = Some(Token::OpenAngle(SingleTokenSpan::new(cursor))),
            '>' => curr_token = Some(Token::ClosedAngle(SingleTokenSpan::new(cursor))),
            ':' => curr_token = Some(Token::Colon(SingleTokenSpan::new(cursor))),
            ';' => curr_token = Some(Token::Semi(SingleTokenSpan::new(cursor))),
            ',' => curr_token = Some(Token::Comma(SingleTokenSpan::new(cursor))),
            '#' => curr_token = Some(Token::Hashtag(SingleTokenSpan::new(cursor))),
            '\'' => curr_token = Some(Token::Apostrophe(SingleTokenSpan::new(cursor))),
            '?' => curr_token = Some(Token::Question(SingleTokenSpan::new(cursor))),
            '_' => curr_token = Some(Token::Underscore(SingleTokenSpan::new(cursor))),
            '.' => curr_token = Some(Token::Dot(SingleTokenSpan::new(cursor))),
            '/' => {
                match input[cursor + 1] {
                    '/' => {
                        let span_start = cursor;
                        cursor += 2;
                        let mut buffer = String::new();
                        while input.len() > cursor && input[cursor] != '\n' {
                            buffer.push(input[cursor]);
                            cursor += 1;
                        }
                        curr_token = Some(Token::Comment(Span::multi_token(span_start, cursor), buffer));
                    },
                    '=' => curr_token = Some(Token::BinOp(Span::multi_token(cursor, cursor + 1), BinOp::DivEq)),
                    _ => curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Div)),
                }
            },
            '+' => {
                if input[cursor + 1] == '=' {
                    curr_token = Some(Token::BinOp(Span::multi_token(cursor, cursor + 1), BinOp::AddEq));
                } else {
                    curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Add));
                }
            },
            '-' => {
                if input[cursor + 1] == '=' {
                    curr_token = Some(Token::BinOp(Span::multi_token(cursor, cursor + 1), BinOp::SubEq));
                } else {
                    curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Sub));
                }
            },
            '*' => {
                if input[cursor + 1] == '=' {
                    curr_token = Some(Token::BinOp(Span::multi_token(cursor, cursor + 1), BinOp::MulEq));
                } else {
                    curr_token = Some(Token::BinOp(Span::single_token(cursor), BinOp::Mul));
                }
            },
            ('\r' | '\n') => {}, // this is a noop
            _ => {
                curr_token = Some(Token::Invalid(SingleTokenSpan::new(cursor), curr));
            },
        }
        if let Some(token) = curr_token.take() {
            tokens.push(token);
        }
        cursor += 1;
    }

    /*

    ****************************************************************
                                OLD

    */

    /*
    let mut buffer_type = BufferType::None;
    let mut buffer: Option<Vec<char>> = None;
    for x in input.chars().enumerate() {
        let mut curr_token = None;
        match x.1 {
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
                curr_token = Some(Token::Quote(SingleTokenSpan::new(x.0)));
            },
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
            '(' => curr_token = Some(Token::OpenParen(SingleTokenSpan::new(x.0))),
            ')' => curr_token = Some(Token::ClosedParen(SingleTokenSpan::new(x.0))),
            '{' => curr_token = Some(Token::OpenCurly(SingleTokenSpan::new(x.0))),
            '}' => curr_token = Some(Token::ClosedCurly(SingleTokenSpan::new(x.0))),
            '[' => curr_token = Some(Token::OpenBracket(SingleTokenSpan::new(x.0))),
            ']' => curr_token = Some(Token::ClosedBracket(SingleTokenSpan::new(x.0))),
            '<' => curr_token = Some(Token::OpenAngle(SingleTokenSpan::new(x.0))),
            '>' => curr_token = Some(Token::ClosedAngle(SingleTokenSpan::new(x.0))),
            ':' => curr_token = Some(Token::Colon(SingleTokenSpan::new(x.0))),
            ';' => curr_token = Some(Token::Semi(SingleTokenSpan::new(x.0))),
            ',' => curr_token = Some(Token::Comma(SingleTokenSpan::new(x.0))),
            '#' => curr_token = Some(Token::Hashtag(SingleTokenSpan::new(x.0))),
            '\'' => curr_token = Some(Token::Apostrophe(SingleTokenSpan::new(x.0))),
            '?' => curr_token = Some(Token::Question(SingleTokenSpan::new(x.0))),
            '_' => curr_token = Some(Token::Underscore(SingleTokenSpan::new(x.0))),
            '.' => {
                if buffer_type == BufferType::StrLit || (buffer_type == BufferType::NumLit &&
                    buffer.map_or(false, |buffer| !buffer.contains(&'.') && !buffer.is_empty())) {
                    if let Some(buffer) = &mut buffer {
                        buffer.push('.');
                    }
                } else if buffer_type == BufferType::None {
                    curr_token = Some(Token::Dot(SingleTokenSpan::new(x.0)));
                } else {
                    // FIXME: error!
                }
            },
            ('\r' | '\n') => {}, // this is a noop
            _ => curr_token = Some(Token::Invalid(SingleTokenSpan::new(x.0), x.1)),
        }
        if let Some(token) = curr_token.take() {
            tokens.push(token);
        }
        tokens.push(Token::EOF(SingleTokenSpan::new(input.chars().count()))); // FIXME: don't go over the entire input again - IMPROVE this!
    }*/

    /*

    ******************************************************
                            PERSISTENT
     */
    tokens.push(Token::EOF(SingleTokenSpan::new(input.as_slice().len())));

    if diagnostics_builder.is_empty() {
       Ok(tokens)
    } else {
        Err(diagnostics_builder)
    }
}
