use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::{FixedTokenSpan, Span};
use crate::lexer;
use crate::lexer::token::{BinOp, Token, TokenType};
use crate::parser::ast::{AstNode, BinaryExprNode, Block, BlockModifiers, CallExprNode, ConstValNode, Crate, FunctionHeader, FunctionModifiers, FunctionNode, ItemKind, LAssign, LDecAssign, LocalAssign, NumberType, StaticValNode, Stmt, StmtKind, StructDef, StructFieldDef, TraitDef};
use crate::parser::attrs::{Constness, Mutability, Visibility};
use crate::parser::keyword::Keyword;
use crate::parser::token_stream::TokenStream;
use std::fs;

// converts a stream of tokens into an ast
// (a compiler is just a program that operates on data
// and the parser transforms a stream of tokens into data we can use later)

pub struct Parser {
    token_stream: TokenStream,
    curr: Token,
    diagnostics: DiagnosticBuilder,
}

impl Parser {
    // FIXME: see: https://www.youtube.com/watch?v=4m7ubrdbWQU

    pub fn new(mut token_stream: TokenStream) -> Self {
        let curr = token_stream.get_next_and_advance().unwrap().clone();
        Self {
            token_stream,
            curr,
            diagnostics: DiagnosticBuilder::new(),
        }
    }

    pub fn parse_crate(&mut self) -> Result<Crate, ()> {
        let mut items = vec![];
        while self.curr.to_type() != TokenType::EOF && self.token_stream.can_advance() {
            // FIXME: this loop runs indefinitely!
            println!("in loop!");
            match self.parse_item() {
                Ok(val) => {
                    if let Some(item) = val {
                        items.push(item);
                    } else {
                        self.advance(); // FIXME: is this correct?
                    }
                    /*if let Some(node) = val {
                        self.ast.push(node);
                    } else {
                        self.advance(); // FIXME: is this correct?
                    }*/
                }
                Err(_) => {
                    self.advance(); // FIXME: is this correct?
                                    // FIXME: insert error into diagnostics builder
                }
            }
        }
        Ok(Crate { items })
    }

    /*
    pub fn parse_all(&mut self) {
        while self.curr.to_type() != TokenType::EOF && self.token_stream.can_advance() {
            // FIXME: this loop runs indefinitely!
            println!("in loop!");
            match self.parse_entry() {
                Ok(val) => {
                    self.ast.push(val);
                    /*if let Some(node) = val {
                        self.ast.push(node);
                    } else {
                        self.advance(); // FIXME: is this correct?
                    }*/
                }
                Err(_) => {
                    self.advance(); // FIXME: is this correct?
                                    // FIXME: insert error into diagnostics builder
                }
            }
        }
    }*/

    fn parse_number_expr(&mut self) -> Option<AstNode> {
        if let Token::NumLit(_, content) = &self.curr {
            let ret = Some(AstNode::Number(NumberType::F64(
                content.parse::<f64>().unwrap(),
            ))); // FIXME: do proper parsing of numbers
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

    fn parse_ident(&mut self) -> Option<(Span, String)> {
        if let Token::Ident(sp, val) = &self.curr {
            let ret = Some((*sp, val.clone()));
            self.advance();
            ret
        } else {
            None
        }
    }

    /// assumes the let keyword was already skipped
    fn parse_let(&mut self) -> Result<StmtKind, ()> {
        let name = self.parse_ident();
        if let Some((_, name)) = name {
            let ty = if self.eat(TokenType::Colon) {
                self.parse_ident().map(|x| x.1)
            } else {
                None
            };
            if let Token::BinOp(_, BinOp::Eq) = self.curr {
                self.advance();
                let val = self.parse_expr()?;
                if val.is_none() {
                    return Err(());
                }

                if !self.eat(TokenType::Semi) {
                    return Err(());
                }

                return Ok(StmtKind::LocalAssign(LocalAssign::DecAssign(LDecAssign {
                    ty,
                    val: LAssign {
                        name,
                        val: val.unwrap(),
                    },
                })));
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    /*
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
    }*/

    fn parse_function_header(&mut self) -> Result<FunctionHeader, ()> {
        // skip the `fn` keyword
        self.advance();
        if let Token::Ident(_, name) = &self.curr {
            println!("ident!");
            let name = name.clone();
            self.advance();
            if !self.eat(TokenType::OpenParen) {
                return Err(());
            }
            println!("open paren {}", name);
            let mut args = vec![];
            while let Some(param) = self.parse_param()? {
                args.push(param);
                if !self.eat(TokenType::Comma) {
                    break;
                }
            }

            if !self.eat(TokenType::ClosedParen) {
                return Err(());
            }

            let ret = if self.eat(TokenType::Arrow) {
                let val = self.parse_ident();
                val.map(|x| x.1)
            } else {
                None
            };

            Ok(FunctionHeader {
                name,
                args,
                ret,
            })
        } else {
            Err(()) // FIXME: return error!
        }
    }

    fn parse_function(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        let header = self.parse_function_header()?;
        let body = self.parse_block_no_attr()?;

        Ok(ItemKind::FunctionDef(Box::new(FunctionNode {
            modifiers: FunctionModifiers {
                constness: Constness::Undefined,
                visibility: visibility.unwrap_or(Visibility::Private),
            },
            header,
            body,
        })))
    }

    fn parse_param(&mut self) -> Result<Option<(String, String)>, ()> {
        if let Token::Ident(_, name) = &self.curr {
            let name = name.clone();
            self.advance();

            if !self.eat(TokenType::Colon) {
                return Err(());
            }

            if let Token::Ident(_, ty) = &self.curr {
                let ret = Some((name, ty.clone()));
                self.advance();
                Ok(ret)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
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
                    return Some(AstNode::CallExpr(CallExprNode { callee: name, args }));
                }
            }
        }
        None
    }

    fn parse_bin_op(&mut self) -> Result<Option<AstNode>, ()> {
        let lhs = self.parse_primary()?;

        if let Some(lhs) = lhs {
            self.parse_bin_op_rhs(0, lhs)
        } else {
            Err(())
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
                        rhs = rhs.map(|rhs| {
                            self.parse_bin_op_rhs(bin_op.precedence() + 1, rhs)
                                .unwrap()
                                .unwrap()
                        });
                        if rhs.is_none() {
                            return Err(()); // FIXME: is this correct?
                        }
                    }
                } else {
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
        if self.eat_kw(Keyword::Pub) {
            Some(Visibility::Public)
        } else {
            None
        }
    }

    fn parse_stmt_or_expr(&mut self) -> Result<StmtKind, ()> {
        // handle `let x = y;`
        if self.eat_kw(Keyword::Let) {
            return self.parse_let();
        }
        // FIXME: handle `x = y;`
        let expr = self.parse_expr()?;
        if self.eat(TokenType::Semi) {
            if let Some(expr) = expr {
                return Ok(StmtKind::Semi(expr));
            }
        }
        Ok(expr.map_or(StmtKind::Empty, |node| StmtKind::Expr(node)))
    }

    fn parse_block_no_attr(&mut self) -> Result<Block, ()> {
        if !self.eat(TokenType::OpenCurly) {
            return Err(());
        }
        let mut stmts = vec![];
        while self.curr.to_type() != TokenType::ClosedCurly {
            let combined = self.parse_stmt_or_expr()?;
            match combined {
                StmtKind::Item(_) => {} // FIXME: err
                StmtKind::Semi(_) | StmtKind::LocalAssign(_) | StmtKind::Empty => {
                    stmts.push(combined);
                }
                StmtKind::Expr(_) => {
                    // there can only be a single expr(without a trailing semi in a block)
                    // and that's at its end
                    stmts.push(combined);
                    break;
                }
            }
        }

        if self.eat(TokenType::ClosedCurly) {
            Ok(Block {
                modifiers: BlockModifiers {},
                stmts,
            })
        } else {
            Err(())
        }
    }

    fn parse_mutability(&mut self) -> Option<Mutability> {
        if self.eat_kw(Keyword::Mut) {
            Some(Mutability::Mut)
        } else {
            None
        }
    }

    fn parse_static(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip `static` keyword
        self.advance();
        let mutability = self.parse_mutability();

        if let Token::Ident(_, name) = &self.curr {
            let name = name.clone();
            self.advance();
            let ty = if self.eat(TokenType::Colon) {
                if let Token::Ident(_, ty) = &self.curr {
                    let ty = ty.clone();
                    self.advance();
                    Ok(ty)
                } else {
                    // FIXME: error!
                    Err(())
                }
            } else {
                // FIXME: error!
                Err(())
            }?;

            let rhs = self.parse_bin_op_rhs(0, AstNode::Ident(name))?.unwrap();
            if !self.eat(TokenType::Semi) {
                return Err(());
            }

            Ok(ItemKind::StaticVal(Box::new(StaticValNode {
                ty,
                mutability,
                val: rhs,
                visibility,
            })))
        } else {
            Err(())
        }
    }

    fn parse_const(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip the `const` keyword
        self.advance();

        if let Token::Ident(_, name) = &self.curr {
            let name = name.clone();
            self.advance();
            let ty = if self.eat(TokenType::Colon) {
                if let Token::Ident(_, ty) = &self.curr {
                    let ty = ty.clone();
                    self.advance();
                    Ok(ty)
                } else {
                    // FIXME: error!
                    Err(())
                }
            } else {
                // FIXME: error!
                Err(())
            }?;

            let rhs = self.parse_bin_op_rhs(0, AstNode::Ident(name))?.unwrap();
            if !self.eat(TokenType::Semi) {
                return Err(());
            }

            Ok(ItemKind::ConstVal(Box::new(ConstValNode {
                ty,
                val: rhs,
                visibility,
            })))
        } else {
            Err(())
        }
    }

    fn parse_struct_def(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip the `struct` keyword
        self.advance();
        if let Some((_, name)) = self.parse_ident() {
            if !self.eat(TokenType::OpenCurly) {
                return Err(());
            }

            fn parse_param_with_vis(parser: &mut Parser) -> Result<Option<(Visibility, String, String)>, ()> {
                let vis = parser.parse_visibility();

                let param = parser.parse_param()?;
                if let Some(param) = param {
                    Ok(Some((vis.unwrap_or(Visibility::Private), param.0, param.1)))
                } else {
                    if vis.is_none() {
                        Ok(None)
                    } else {
                        Err(())
                    }
                }
            }

            let mut fields = vec![];
            while let Some((visibility, name, ty)) = parse_param_with_vis(self)? {
                fields.push(StructFieldDef {
                    visibility,
                    name,
                    ty,
                });
                if !self.eat(TokenType::Comma) {
                    break;
                }
            }

            if !self.eat(TokenType::ClosedCurly) {
                return Err(());
            }

            Ok(ItemKind::StructDef(StructDef {
                visibility: visibility.unwrap_or(Visibility::Private),
                name,
                fields,
            }))
        } else {
            Err(())
        }
    }

    fn parse_trait_def(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip the `trait` keyword
        self.advance();
        if let Some((_, name)) = self.parse_ident() {
            if !self.eat(TokenType::OpenCurly) {
                return Err(());
            }

            let mut methods = vec![];
            while self.check_kw(Keyword::Fn) {
                let header = self.parse_function_header()?;
                
                if !self.eat(TokenType::Semi) {
                    return Err(());
                }
                methods.push(header);
            }

            if !self.eat(TokenType::ClosedCurly) {
                return Err(());
            }

            Ok(ItemKind::TraitDef(TraitDef {
                visibility: visibility.unwrap_or(Visibility::Private),
                name,
                methods,
            }))
        } else {
            Err(())
        }
    }

    fn parse_glob(&mut self) -> Result<Option<ItemKind>, ()> {
        let visibility = self.parse_visibility()/*.unwrap_or(Visibility::Private)*/;

        match self.curr {
            Token::Ident(_, _) => {
                // FIXME: try to recover
            }
            Token::Keyword(_, kw) => {
                return match kw {
                    Keyword::Pub => {
                        // FIXME: error
                        Err(())
                    }
                    Keyword::Static => self.parse_static(visibility).map(|item| Some(item)),
                    Keyword::Const => {
                        // FIXME: distinguish between const func and const value!
                        Err(())
                    }
                    Keyword::Rt => Err(()), // FIXME: ?
                    Keyword::Fn => self.parse_function(visibility).map(|item| Some(item)),
                    Keyword::Enum => Err(()),
                    Keyword::Struct => self.parse_struct_def(visibility).map(|item| Some(item)),
                    Keyword::Mod => Err(()),
                    Keyword::Impl => Err(()),
                    Keyword::Async => Err(()),
                    Keyword::Unsafe => Err(()),
                    Keyword::Extern => Err(()),
                    Keyword::Trait => self.parse_trait_def(visibility).map(|item| Some(item)),
                    Keyword::Type => Err(()),
                    _ => Ok(None), // FIXME: error
                };
            }
            Token::StrLit(_, _) => {
                // FIXME: try to recover
            }
            Token::NumLit(_, _) => {
                // FIXME: try to recover
            }
            Token::OpenParen(_) => {}
            Token::Colon(_) => {
                // FIXME: MAYBE try to recover
            }
            Token::Comment(_, _) => {}
            _ => {}
        }
        Ok(None)
    }

    fn parse_primary(&mut self) -> Result<Option<AstNode>, ()> {
        println!("curr: {:?}", self.curr);
        match &self.curr {
            Token::Ident(_, content) => {
                if self
                    .token_stream
                    .look_ahead(1, |token| token.to_type() == TokenType::OpenParen)
                {
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
            _ => Err(()),
        }
    }

    /// parses an expression for example in the body of a method
    fn parse_expr(&mut self) -> Result<Option<AstNode>, ()> {
        if self.check(TokenType::OpenCurly) {
            return self
                .parse_block_no_attr()
                .map(|block| Some(AstNode::Block(block)));
        }
        self.parse_bin_op()
    }

    fn parse_item(&mut self) -> Result<Option<ItemKind>, ()> {
        match self.curr {
            Token::Keyword(_, _) => self.parse_glob(),
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
            _ => Err(()),
        }
    }

    fn check(&self, token: TokenType) -> bool {
        self.curr.to_type() == token
    }

    fn check_kw(&self, kw: Keyword) -> bool {
        if let Token::Keyword(_, actual) = self.curr {
            actual == kw
        } else {
            false
        }
    }

    fn eat(&mut self, token: TokenType) -> bool {
        if self.curr.to_type() == token {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, kw: Keyword) -> bool {
        if let Token::Keyword(_, curr_kw) = self.curr {
            if kw == curr_kw {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) {
        if let Some(next) = self.token_stream.get_next() {
            self.curr = next.clone();
        } else {
            self.curr = EOF_TOKEN;
        }
        self.token_stream.advance();
    }
}



const EOF_TOKEN: Token = Token::EOF(FixedTokenSpan::new(usize::MAX));

#[cfg(test)]
fn test_file(path: String, assumed: Box<dyn Fn(Vec<Token>, Crate) -> bool>) -> bool {
    let file = fs::read_to_string(path).unwrap();
    let lexed = lexer::lex(file).unwrap();
    let lexed_cloned = lexed.clone();
    let mut token_stream = TokenStream::new(lexed);
    let mut parser = Parser::new(token_stream);
    let krate = parser.parse_crate().unwrap();
    assumed(lexed_cloned, krate)
}

#[test]
fn test_func() {
    assert!(test_file(
        String::from("tests/func.tf"),
        Box::new(|tokens, krate| tokens.len() == 33 && krate.items.len() == 2)
    ));
}

#[test]
fn test_static() {
    assert!(test_file(
        String::from("tests/static.tf"),
        Box::new(|tokens, krate| tokens.len() == 24 && krate.items.len() == 2)
    ));
}

#[test]
fn test_struct() {
    assert!(test_file(
        String::from("tests/struct.tf"),
        Box::new(|tokens, krate| tokens.len() == 19 && krate.items.len() == 2)
    ));
}

#[test]
fn test_trait() {
    assert!(test_file(
        String::from("tests/trait.tf"),
        Box::new(|tokens, krate| tokens.len() == 29 && krate.items.len() == 2)
    ));
}
