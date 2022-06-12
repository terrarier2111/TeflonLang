use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::SingleTokenSpan;
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::{AstNode, BinaryExprNode, CallExprNode, FunctionHeaderNode, FunctionModifiers, FunctionNode, NumberType};
use crate::parser::attrs::Visibility;
use crate::parser::keyword::Keyword;
use crate::parser::token_stream::TokenStream;

pub struct Parser {
    token_stream: TokenStream,
    curr: Token,
    pub ast: Vec<AstNode>,
    diagnostics: DiagnosticBuilder,
}

impl Parser {

    // FIXME: see: https://www.youtube.com/watch?v=4m7ubrdbWQU

    pub fn new(mut token_stream: TokenStream) -> Self {
        let curr = token_stream.get_next_and_advance().unwrap().clone();
        Self {
            token_stream,
            curr,
            ast: vec![],
            diagnostics: DiagnosticBuilder::new(),
        }
    }

    pub fn parse_all(&mut self) {
        while self.curr.to_type() != TokenType::EOF && self.token_stream.can_advance() { // FIXME: this loop runs indefinitely!
            println!("in loop!");
            match self.parse_expr() {
                Ok(val) => {
                    if let Some(node) = val {
                        self.ast.push(node);
                    } else {
                        self.advance(); // FIXME: is this correct?
                    }
                },
                Err(_) => {
                    self.advance(); // FIXME: is this correct?
                    // FIXME: insert error into diagnostics builder
                }
            }
        }
    }

    fn parse_number_expr(&mut self) -> Option<AstNode> {
        if let Token::NumLit(_, content) = &self.curr {
            let ret = Some(AstNode::Number(NumberType::F64(content.parse::<f64>().unwrap()))); // FIXME: do proper parsing of numbers
            self.advance();
            ret
        } else {
            None
        }
    }

    fn parse_paren_expr(&mut self) -> Option<AstNode> {
        if !self.eat(TokenType::OpenParen) {
            return None;
        }
        if let Ok(expr) = self.parse_expr() {
            if self.eat(TokenType::ClosedParen) {
                return expr;
            }
        }
        None
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
            let content = content.clone();
            self.advance();
            if self.eat(TokenType::OpenParen) {
                // let args = self.parse_comma_separated(); // FIXME: parse a comma separated types list instead of a comma separated expr list
                let mut args = vec![];
                if let Some(param) = self.parse_param() {
                    args.push(param);
                    let mut extra_comma = false;
                    let mut emitted_err = false;
                    while self.eat(TokenType::Comma) {
                        if let Some(item) = self.parse_param() {
                            args.push(item);
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

                if self.eat(TokenType::OpenParen) {
                    Some(FunctionHeaderNode {
                        name: content,
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
            let name = name.clone();
            self.advance();
            if self.eat(TokenType::Colon) {
                if let Some(Token::Ident(_, ty)) = self.token_stream.get_next() {
                    let ret = Some((name, ty.clone()));
                    self.advance();
                    ret
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
        if let Ok(Some(item)) = self.parse_expr() {
            ret.push(item);
            let mut extra_comma = false;
            let mut emitted_err = false;
            while self.eat(TokenType::Comma) {
                if let Ok(Some(item)) = self.parse_expr() {
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
            let name = name.clone();
            if self.eat(TokenType::OpenParen) {
                let args = self.parse_comma_separated();
                if self.eat(TokenType::ClosedParen) {
                    Some(AstNode::CallExpr(CallExprNode {
                        callee: name,
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

    fn parse_bin_op(&mut self) -> Result<Option<AstNode>, ()> {
        let lhs = self.parse_primary()?;

        if let Some(lhs) = lhs {
            self.parse_bin_op_rhs(0, lhs)
        } else {
            Err(())
        }
    }

    fn get_token_precedence(&self) -> Option<usize> {
        if let Token::BinOp(_, bin_op) = &self.curr {
            Some(bin_op.precedence())
        } else {
            None
        }
    }

    fn parse_bin_op_rhs(&mut self, prec: usize, mut lhs: AstNode) -> Result<Option<AstNode>, ()> {
        // If this is a binop, find its precedence.
        loop {
            let bin_op = if let Token::BinOp(_, bin_op) = &self.curr {
                Some(*bin_op)
            } else {
                None
            };

            // If this is a binop that binds at least as tightly as the current binop,
            // consume it, otherwise we are done.
            if let Some(bin_op) = bin_op {
                if bin_op.precedence() < prec {
                    return Ok(Some(lhs));
                }
            } else {
                return Ok(Some(lhs));
            }
            let bin_op = bin_op.unwrap();
            self.eat(TokenType::BinOp);

            let mut rhs = self.parse_primary()?;

            if rhs.is_some() {
                // If BinOp binds less tightly with RHS than the operator after RHS, let
                // the pending operator take RHS as its LHS.
                let next_bin_op = if let Token::BinOp(_, bin_op) = &self.curr {
                    Some(*bin_op)
                } else {
                    None
                };

                if let Some(next_bin_op) = next_bin_op {
                    if bin_op.precedence() < next_bin_op.precedence() {
                        /*return Ok(Some(AstNode::BinaryExpr(BinaryExprNode {
                            lhs,
                            rhs,
                            op: bin_op,
                        })));*/
                        // let prev_rhs = rhs.take();
                        rhs = rhs.map(|rhs| self.parse_bin_op_rhs(bin_op.precedence() + 1, rhs).unwrap().unwrap());
                        if rhs.is_none() {
                            return Err(()); // FIXME: is this correct?
                        }
                    }
                } else {
                    // return Ok(Some(lhs));
                    return Ok(Some(AstNode::BinaryExpr(Box::new(BinaryExprNode {
                        lhs,
                        rhs: rhs.take().unwrap(),
                        op: bin_op,
                    }))));
                }

                lhs = AstNode::BinaryExpr(Box::new(BinaryExprNode {
                    lhs,
                    rhs: rhs.take().unwrap(),
                    op: bin_op,
                }))
            }
        }
    }

    /*
    fn parse_pub(&mut self) -> Result<Option<AstNode>, ()> {

    }

    fn parse_kw_glob(&mut self, prev: Option<Vec<Keyword>>) -> Result<Option<AstNode>, ()> {
        if let Some(curr) = self.curr {
            match curr {
                Token::Keyword(_, kw) => {
                    match kw {
                        Keyword::Pub => {
                            // if prev is not empty, error!
                            // struct/enum/function/constant/static definition
                        },
                        Keyword::Static => {
                            // variable
                        },
                        //Keyword::Rt => {}
                        Keyword::Fn => {}
                        Keyword::Enum => {}
                        Keyword::Struct => {}
                        //!Keyword::Mod => {}
                        Keyword::Impl => {}
                        Keyword::Async => {}
                        Keyword::Unsafe => {}
                        //!Keyword::Extern => {}
                        Keyword::Trait => {}
                        Keyword::Type => {}
                        _ => Err(()),
                    }
                },
                // Token::StrLit(_, _) => {}
                //!Token::OpenCurly(_) => {}
                // Token::OpenBracket(_) => {}
                // Token::Eq(_) => {}
                // Token::Colon(_) => {}
                //!Token::Semi(_) => {}
                // Token::Apostrophe(_) => {}
                // Token::OpenAngle(_) => {}
                // Token::Star(_) => {}
                // Token::Question(_) => {}
                // Token::Underscore(_) => {}
                Token::Comment(_, _) => Ok(None),
                Token::EOF(_) => Err(()),
                _ => Err(())
            }
        } else {
            Err(())
        }
    }*/

    fn parse_visibility(&mut self) -> Option<Visibility> {
        if let Token::Keyword(_, kw) = &self.curr {
            match kw {
                Keyword::Pub => {
                    self.advance();
                    Some(Visibility::Public)
                },
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse_glob(&mut self) -> Result<Option<AstNode>, ()> {
        let visibility = self.parse_visibility()/*.unwrap_or(Visibility::Private)*/;

        /*match self.curr {
            Token::Ident(_, _) => {
                // FIXME: try to recover
            }
            Token::Keyword(_, _) => {}
            Token::StrLit(_, _) => {
                // FIXME: try to recover
            }
            Token::NumLit(_, _) => {
                // FIXME: try to recover
            }
            Token::OpenParen(_) => {}
            Token::OpenCurly(_) => {}
            Token::OpenBracket(_) => {}
            Token::Eq(_) => {
                // FIXME: MAYBE try to recover
            }
            Token::Colon(_) => {
                // FIXME: MAYBE try to recover
            }
            Token::Comment(_, _) => {}
        }*/
        Ok(None)
    }

    fn parse_primary(&mut self) -> Result<Option<AstNode>, ()> {
        println!("curr: {:?}", self.curr);
        match &self.curr {
            Token::Ident(_, content) => {
                if self.token_stream.look_ahead(1, |token| token.to_type() == TokenType::OpenParen) {
                    Ok(self.parse_call()) // FIXME: handle errors properly!
                    /*} else if self.token_stream.look_ahead(1, |token| token.to_type() == TokenType::Dot) {
                        // FIXME: parse field access/struct method call
                        */
                } else {
                    // FIXME: handle the rest!
                    let content = content.clone();
                    self.advance();
                    Ok(Some(AstNode::Ident(content)))
                }
            }
            //#!Token::Keyword(_, _) => {}
            // Token::StrLit(_, _) => {}
            Token::NumLit(_, _) => Ok(self.parse_number_expr()),
            Token::OpenParen(_) => Ok(self.parse_paren_expr()),
            //#!Token::OpenCurly(_) => {}
            // Token::OpenBracket(_) => {}
            // Token::Eq(_) => {}
            // Token::Colon(_) => {}
            //#!Token::Semi(_) => {}
            // Token::Apostrophe(_) => {}
            // Token::OpenAngle(_) => {}
            // Token::Star(_) => {}
            // Token::Question(_) => {}
            // Token::Underscore(_) => {}
            Token::Comment(_, _) => Ok(None),
            Token::EOF(_) => Err(()),
            _ => Err(())
        }
    }

    /// parses an expression for example in the body of a method
    fn parse_expr(&mut self) -> Result<Option<AstNode>, ()> {
        self.parse_bin_op()
    }

    fn parse_entry(&mut self) -> Result<Option<AstNode>, ()> {
        /*if let Some(curr) = self.curr {
            match curr {
                Token::Keyword(_, kw) => {

                },
                // Token::StrLit(_, _) => {}
                //#!Token::OpenCurly(_) => {}
                // Token::OpenBracket(_) => {}
                // Token::Eq(_) => {}
                // Token::Colon(_) => {}
                //#!Token::Semi(_) => {}
                // Token::Apostrophe(_) => {}
                // Token::OpenAngle(_) => {}
                // Token::Star(_) => {}
                // Token::Question(_) => {}
                // Token::Underscore(_) => {}
                Token::Comment(_, _) => Ok(None),
                Token::EOF(_) => Err(()),
                _ => Err(())
            }
        } else {
            Err(())
        }*/
        Ok(None)
    }

    fn eat(&mut self, token: TokenType) -> bool {
        if self.curr.to_type() == token {
            if let Some(next) = self.token_stream.get_next() {
                self.curr = next.clone();
            } else {
                self.curr = Token::EOF(SingleTokenSpan::new(usize::MAX));
            }
            self.token_stream.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if let Some(next) = self.token_stream.get_next() {
            self.curr = next.clone();
        } else {
            self.curr = Token::EOF(SingleTokenSpan::new(usize::MAX));
        }
        self.token_stream.advance();
    }

}
