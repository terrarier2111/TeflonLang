use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::{AstNode, CallExprNode, FunctionHeaderNode, FunctionModifiers, FunctionNode, NumberType};
use crate::parser::keyword::Keyword;
use crate::parser::token_stream::TokenStream;

pub struct Parser<'a> {
    token_stream: TokenStream,
    curr: Option<&'a Token>,
    ast: Vec<AstNode>,
}

impl Parser<'_> {

    // FIXME: see: https://www.youtube.com/watch?v=4m7ubrdbWQU


    fn parse_number_expr(&mut self) -> Option<AstNode> {
        if let Some(Token::NumLit(span, content)) = self.token_stream.get_next() {
            Some(AstNode::Number(NumberType::F64(0.0))) // FIXME: do proper parsing of numbers
        } else {
            None
        }
    }

    fn parse_function_header(&mut self) -> Option<AstNode> {
        /*if let Some(curr) = self.curr {
            match curr {
                Token::Keyword(_, kw) => {
                    match kw {
                        Keyword::Pub => {}
                        Keyword::Rt => {}
                        Keyword::Fn => {
                            // FIXME: parse: IDENT(ARGS)
                        },
                        // Keyword::Async => {}
                        // Keyword::Unsafe => {}
                        // Keyword::Extern => {}
                        _ => None, // FIXME: do proper error recovery!
                    }
                }
                _ => None,
            }
        } else {
            None
        }*/
        None
    }

    fn parse_function_header_2(&mut self, modifiers: FunctionModifiers) -> Option<FunctionHeaderNode> {
        if let Some(Token::Ident(_, content)) = self.token_stream.get_next() {
            self.token_stream.advance();
            if self.token_stream.eat(TokenType::OpenParen) {
                // let args = self.parse_comma_separated(); // FIXME: parse a comma separated types list instead of a comma separated expr list
                let mut args = vec![];
                if let Some(param) = self.parse_param() {
                    ret.push(param);
                    let mut extra_comma = false;
                    let mut emitted_err = false;
                    while self.eat(TokenType::Comma) {
                        if let Some(item) = self.parse_param() {
                            ret.push(item);
                        } else {
                            if extra_comma {
                                if !emitted_err {
                                    // FIXME: handle error!
                                    emitted_err = true;
                                }
                                // break;
                            } else {
                                extra_comma = true;
                            }
                        }
                    }
                }

                if self.token_stream.eat(TokenType::OpenParen) {
                    Some(FunctionHeaderNode {
                        name: content.clone(),
                        modifiers,
                        args,
                    })
                } else {
                    None // FIXME: return error!
                }
            } else {
                None // FIXME: return error!
            }
        } else {
            None // FIXME: return error!
        }
    }

    fn parse_param(&mut self) -> Option<(String, String)> {
        if let Some(Token::Ident(_, name)) = self.token_stream.get_next() {
            self.token_stream.advance();
            if self.eat(TokenType::Colon) {
                if let Some(Token::Ident(_, ty)) = self.token_stream.get_next() {
                    self.token_stream.advance();
                    Some((name.clone(), ty.clone()))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse_comma_separated(&mut self) -> Vec<AstNode> {
        let mut ret = vec![];
        if let Some(item) = self.parse_expr() {
            ret.push(item);
            let mut extra_comma = false;
            let mut emitted_err = false;
            while self.eat(TokenType::Comma) {
                if let Some(item) = self.parse_expr() {
                    ret.push(item);
                } else {
                    if extra_comma {
                        if !emitted_err {
                            // FIXME: handle error!
                            emitted_err = true;
                        }
                        // break;
                    } else {
                        extra_comma = true;
                    }
                }
            }
        }
        ret
    }

    fn parse_call(&mut self) -> Option<AstNode> {
        if let Some(Token::Ident(_, name)) = self.token_stream.get_next() {
            if self.eat(TokenType::OpenParen) {
                let args = self.parse_comma_separated();
                if self.eat(TokenType::ClosedParen) {
                    Some(AstNode::CallExpr(CallExprNode {
                        callee: name.clone(),
                        args
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// parses an expression for example in the body of a method
    fn parse_expr(&mut self) -> Option<AstNode> {
        if let Some(curr) = self.curr {
            match curr {
                Token::Ident(_, _) => {
                    if self.token_stream.look_ahead(1, |token| token.to_type() == TokenType::OpenParen) {
                        self.ast.push(self.parse_call().unwrap()); // FIXME: handle errors properly!
                    } else if self.token_stream.look_ahead(1, |token| token.to_type() == TokenType::Dot) {
                        // FIXME: parse field access/struct method call
                    } else {
                        // FIXME: handle the rest!
                    }
                }
                Token::Keyword(_, _) => {}
                Token::BinOp(_, _) => {}
                Token::StrLit(_, _) => {}
                Token::NumLit(_, _) => {}
                Token::OpenParen(_) => {}
                Token::OpenCurly(_) => {}
                // Token::OpenBracket(_) => {}
                // Token::Eq(_) => {}
                // Token::Colon(_) => {}
                Token::Semi(_) => {}
                Token::Apostrophe(_) => {}
                // Token::OpenAngle(_) => {}
                Token::Star(_) => {}
                Token::Question(_) => {}
                Token::Underscore(_) => {}
                Token::Comment(_, _) => {}
                Token::EOF(_) => {}
                _ => {}
            }
            None
        } else {
            None
        }
    }

    fn parse_entry(&mut self) {

    }

    fn eat(&mut self, token: TokenType) -> bool {
        self.token_stream.eat(token)
    }

}
