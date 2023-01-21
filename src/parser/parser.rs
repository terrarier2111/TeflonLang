use crate::diagnostics::builder::DiagnosticBuilder;
use crate::diagnostics::span::{FixedTokenSpan, Span};
use crate::lexer;
use crate::lexer::token::{BinOp, Token, TokenType};
use crate::parser::ast::{
    ArrayInst, ArrayInstList, ArrayInstShort, ArrayTy, AstNode, BinaryExprNode, Block,
    BlockModifiers, CallExprNode, ConstValNode, Crate, FunctionHeader, FunctionModifiers,
    FunctionNode, Generic, GenericConstant, GenericLifetime, GenericType, ItemKind, LAssign,
    LDecAssign, Lifetime, LocalAssign, NumberType, OwnedTy, RefTy, StaticValNode, Stmt, StmtKind,
    StructConstructor, StructDef, StructFieldDef, AdtImpl, TraitDef, Ty, TyKind, TyOrConstVal,
};
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

// FIXME: should we actually do this here?
// FIXME: check for duplicate parameter/function names

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
                    items.push(val);
                }
                Err(_) => {
                    self.advance(); // FIXME: is this correct?
                                    // FIXME: insert error into diagnostics builder
                    println!("err!!!");
                }
            }
        }
        Ok(Crate {
            items: items.into_boxed_slice(),
        })
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

    fn parse_number_expr(&mut self) -> Result<AstNode, ()> {
        if let Token::NumLit(_, content) = &self.curr {
            let ret = Ok(AstNode::Number(NumberType::F64(
                content.parse::<f64>().unwrap(),
            ))); // FIXME: do proper parsing of numbers
            self.advance();
            ret
        } else {
            Err(())
        }
    }

    fn parse_paren_expr(&mut self) -> Result<AstNode, ()> {
        if !self.eat(TokenType::OpenParen) {
            return Err(());
        }
        let expr = self.parse_expr()?;

        if !self.eat(TokenType::ClosedParen) {
            return Err(());
        }
        Ok(expr)
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
        let mutability = self.parse_mutability();
        let name = self.parse_ident();
        if let Some((_, name)) = name {
            let ty = if self.eat(TokenType::Colon) {
                Some(self.parse_ty()?)
            } else {
                None
            };
            if let Token::BinOp(_, BinOp::Eq) = self.curr {
                self.advance();
                let val = self.parse_expr()?;

                if !self.eat(TokenType::Semi) {
                    return Err(());
                }

                return Ok(StmtKind::LocalAssign(LocalAssign::DecAssign(LDecAssign {
                    mutability,
                    ty,
                    val: LAssign { name, val },
                })));
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn parse_function_header(&mut self) -> Result<FunctionHeader, ()> {
        // skip the `fn` keyword
        self.advance();
        if let Some((_, name)) = self.parse_ident() {
            println!("ident!");
            let generics = self.parse_maybe_generics_definition()?;

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
                let val = self.parse_ty()?;
                Some(val)
            } else {
                None
            };

            Ok(FunctionHeader {
                name,
                generics,
                args: args.into_boxed_slice(),
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

    fn parse_param(&mut self) -> Result<Option<(String, Ty)>, ()> {
        if let Some((_, name)) = self.parse_ident() {
            if !self.eat(TokenType::Colon) {
                return Err(());
            }

            let ty = self.parse_ty()?;
            Ok(Some((name, ty)))
        } else {
            Ok(None)
        }
    }

    fn parse_comma_separated(&mut self) -> Vec<AstNode> {
        let mut ret = vec![];
        while let Ok(item) = self.parse_expr() {
            ret.push(item);
            if !self.eat(TokenType::Comma) {
                break;
            }
        }
        ret
    }

    fn parse_call(&mut self) -> Result<AstNode, ()> {
        if let Some((_, name)) = self.parse_ident() {
            if self.eat(TokenType::OpenParen) {
                let args = self.parse_comma_separated();
                if self.eat(TokenType::ClosedParen) {
                    return Ok(AstNode::CallExpr(CallExprNode {
                        callee: name,
                        args: args.into_boxed_slice(),
                    }));
                }
            }
        }
        Err(())
    }

    fn parse_bin_op(&mut self) -> Result<AstNode, ()> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    fn parse_bin_op_rhs(&mut self, prec: usize, mut lhs: AstNode) -> Result<AstNode, ()> {
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
                    return Ok(lhs);
                }
            } else {
                return Ok(lhs);
            }
            let bin_op = bin_op.unwrap();
            self.eat(TokenType::BinOp);

            let mut rhs = Some(self.parse_primary()?);

            // If BinOp binds less tightly with RHS than the operator after RHS, let
            // the pending operator take RHS as its LHS.
            let next_bin_op = if let Token::BinOp(_, bin_op) = &self.curr {
                Some(*bin_op)
            } else {
                None
            };

            match next_bin_op {
                None => {
                    return Ok(AstNode::BinaryExpr(Box::new(BinaryExprNode {
                        lhs,
                        rhs: rhs.take().unwrap(),
                        op: bin_op,
                    })));
                },
                Some(next_bin_op) => {
                    if bin_op.precedence() < next_bin_op.precedence() {
                        rhs = rhs.map(|rhs| {
                            self.parse_bin_op_rhs(bin_op.precedence() + 1, rhs).unwrap()
                        });
                    }
                    lhs = AstNode::BinaryExpr(Box::new(BinaryExprNode {
                        lhs,
                        rhs: rhs.take().unwrap(),
                        op: bin_op,
                    }));
                }
            }
        }
    }

    fn parse_visibility(&mut self) -> Option<Visibility> {
        if self.eat_kw(Keyword::Pub) {
            Some(Visibility::Public)
        } else {
            None
        }
    }

    fn parse_lt(&mut self) -> Result<Lifetime, ()> {
        if let Ok(Some(lt)) = self.parse_maybe_lt() {
            Ok(lt)
        } else {
            Err(())
        }
    }

    fn parse_maybe_lt(&mut self) -> Result<Option<Lifetime>, ()> {
        if !self.eat(TokenType::Apostrophe) {
            return Ok(None);
        }
        if let Some((_, name)) = self.parse_ident() {
            Ok(Some(match name.as_str() {
                "static" => Lifetime::Static,
                "_" => Lifetime::Inferred,
                _ => Lifetime::Custom(name), // FIXME: disallow lifetimes starting with '_'
            }))
        } else {
            Err(())
        }
    }

    fn parse_maybe_generics_definition(&mut self) -> Result<Box<[Generic]>, ()> {
        if !self.eat(TokenType::OpenAngle) {
            return Ok(Box::new([]));
        }

        let mut generics = vec![];
        while !self.check(TokenType::ClosedAngle) {
            if self.eat_kw(Keyword::Const) {
                if let Some((_, name)) = self.parse_ident() {
                    if !self.eat(TokenType::Colon) {
                        return Err(());
                    }
                    let ty = self.parse_ty()?;
                    generics.push(Generic::Constant(GenericConstant { name, ty }));
                } else {
                    return Err(());
                }
            } else if let Some((_, name)) = self.parse_ident() {
                let traits = if self.eat(TokenType::Colon) {
                    let mut traits = vec![];
                    traits.push(self.parse_ty()?);

                    while self.eat_bin_op(BinOp::Add) {
                        let ty = self.parse_ty()?;
                        traits.push(ty);
                    }

                    traits.into_boxed_slice()
                } else {
                    Box::new([])
                };

                generics.push(Generic::Type(GenericType {
                    name,
                    required_traits: traits,
                }));
            } else {
                let lt = self.parse_lt()?;
                generics.push(Generic::Lifetime(GenericLifetime {
                    lt, // FIXME: support constraints!
                }));
            }

            if !self.eat(TokenType::Comma) {
                break;
            }
        }

        if !self.eat(TokenType::ClosedAngle) {
            return Err(());
        }

        if generics.is_empty() {
            // FIXME: is this check at the right spot?
            return Err(());
        }

        Ok(generics.into_boxed_slice())
    }

    fn parse_maybe_const_generic_vals_and_tys(&mut self) -> Result<Box<[TyOrConstVal]>, ()> {
        if !self.eat(TokenType::OpenAngle) {
            return Ok(Box::new([]));
        }

        self.parse_const_generic_vals_and_tys()
    }

    fn parse_const_generic_vals_and_tys(&mut self) -> Result<Box<[TyOrConstVal]>, ()> {
        // `<` was already skipped
        let mut generics = vec![];

        while !self.check(TokenType::ClosedAngle) {
            let ty_or_expr = self.parse_ty_or_expr(&[TokenType::Comma, TokenType::ClosedAngle])?;

            match ty_or_expr {
                (Some(ty), _) => {
                    // FIXME: try to find a better way do that we don't have to convert the ty
                    // FIXME: opportunistically later if we find out it is infact a const generic val and not a ty
                    generics.push(TyOrConstVal::Ty(ty));
                }
                (_, Some(expr)) => {
                    generics.push(TyOrConstVal::ConstVal(expr));
                }
                _ => unreachable!(),
            }
            if !self.eat(TokenType::Comma) {
                break;
            }
        }

        if !self.eat(TokenType::ClosedAngle) {
            return Err(());
        }

        Ok(generics.into_boxed_slice())
    }

    fn parse_ty_or_expr(
        &mut self,
        next_expected: &[TokenType],
    ) -> Result<(Option<Ty>, Option<AstNode>), ()> {
        if self.check(TokenType::Ident)
            && self.token_stream.look_ahead(1, |token| {
                next_expected.iter().any(|ty| ty == &token.to_type())
                    || token.to_type() == TokenType::OpenAngle
            })
        {
            // ty
            self.parse_ty().map(|ty| (Some(ty), None))
        } else {
            // expr
            self.parse_expr().map(|node| (None, Some(node)))
        }
    }

    fn parse_ty(&mut self) -> Result<Ty, ()> {
        if self.eat(TokenType::And) {
            self.parse_ref_ty().map(|rf| Ty {
                kind: TyKind::Ref(Box::new(rf)),
            })
        } else if self.eat(TokenType::OpenBracket) {
            self.parse_array_ty().map(|array| Ty {
                kind: TyKind::Array(Box::new(array)),
            })
        } else {
            self.parse_owned_ty().map(|owned| Ty {
                kind: TyKind::Owned(Box::new(owned)),
            })
        }
    }

    fn parse_owned_ty(&mut self) -> Result<OwnedTy, ()> {
        if let Some((_, name)) = self.parse_ident() {
            let generics = self.parse_maybe_const_generic_vals_and_tys()?;

            return Ok(OwnedTy { name, generics });
        }
        Err(())
    }

    fn parse_ref_ty(&mut self) -> Result<RefTy, ()> {
        let lt = self.parse_maybe_lt()?;
        let mutability = self.parse_mutability().unwrap_or(Mutability::Immut);
        let ty = self.parse_ty()?;
        Ok(RefTy {
            lt,
            mutability,
            ty: Box::new(ty),
        })
    }

    fn parse_array_ty(&mut self) -> Result<ArrayTy, ()> {
        let ty = self.parse_ty()?;

        let amount = if self.eat(TokenType::Semi) {
            Some(self.parse_number_expr()?)
        } else {
            None
        };

        if !self.eat(TokenType::ClosedBracket) {
            return Err(());
        }

        Ok(ArrayTy { ty, amount })
    }

    fn parse_array_constructor(&mut self) -> Result<AstNode, ()> {
        if !self.eat(TokenType::OpenBracket) {
            return Err(());
        }
        let val = self.parse_bin_op()?;

        let inst = match self.curr.to_type() {
            TokenType::Comma => {
                let mut vals = vec![val];
                // we skip the `,` token
                self.advance();
                while !self.check(TokenType::ClosedBracket) {
                    let val = self.parse_bin_op()?;
                    vals.push(val);

                    if !self.eat(TokenType::Comma) {
                        break;
                    }
                }

                if !self.eat(TokenType::ClosedBracket) {
                    return Err(());
                }

                ArrayInst::List(ArrayInstList {
                    vals: vals.into_boxed_slice(),
                })
            }
            TokenType::ClosedBracket => {
                // we skip the `]` token
                self.advance();
                ArrayInst::List(ArrayInstList {
                    vals: Box::new([val]),
                })
            }
            TokenType::Semi => {
                // we skip the `;` token
                self.advance();
                let cnt = self.parse_number_expr()?;

                if !self.eat(TokenType::ClosedBracket) {
                    return Err(());
                }

                ArrayInst::Short(Box::new(ArrayInstShort { val, amount: cnt }))
            }
            _ => {
                return Err(());
            }
        };

        Ok(AstNode::ArrayInst(inst))
    }

    fn parse_stmt_or_expr(&mut self) -> Result<StmtKind, ()> {
        // handle `let x = y;`
        if self.eat_kw(Keyword::Let) {
            return self.parse_let();
        }
        // FIXME: handle `x = y;`
        let expr = self.parse_expr()?;
        if self.eat(TokenType::Semi) {
            return Ok(StmtKind::Semi(expr));
        }
        Ok(StmtKind::Expr(expr))
    }

    fn parse_block_no_attr(&mut self) -> Result<Block, ()> {
        if !self.eat(TokenType::OpenCurly) {
            return Err(());
        }
        let mut stmts = vec![];
        while self.curr.to_type() != TokenType::ClosedCurly {
            let combined = self.parse_stmt_or_expr()?;
            match combined {
                StmtKind::Item(_) => {
                    return Err(());
                }
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
                stmts: stmts.into_boxed_slice(),
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

        if let Some((_, name)) = self.parse_ident() {
            let ty = if self.eat(TokenType::Colon) {
                self.parse_ty()
            } else {
                // FIXME: error!
                Err(())
            }?;

            let rhs = self.parse_bin_op_rhs(0, AstNode::Ident(name))?;
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

        if let Some((_, name)) = self.parse_ident() {
            let ty = if self.eat(TokenType::Colon) {
                self.parse_ty()
            } else {
                // FIXME: error!
                Err(())
            }?;

            let rhs = self.parse_bin_op_rhs(0, AstNode::Ident(name))?;
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

    fn parse_struct_constructor(&mut self) -> Result<AstNode, ()> {
        if let Some((_, name)) = self.parse_ident() {
            if !self.eat(TokenType::OpenCurly) {
                return Err(());
            }
            let mut fields = vec![];
            while let Some((_, name)) = self.parse_ident() {
                if !self.eat(TokenType::Colon) {
                    return Err(());
                }

                let val = self.parse_expr()?;
                fields.push((name, val));

                if !self.eat(TokenType::Comma) {
                    break;
                }
            }
            if !self.eat(TokenType::ClosedCurly) {
                return Err(());
            }
            return Ok(AstNode::StructConstructor(StructConstructor {
                name,
                fields: fields.into_boxed_slice(),
            }));
        } else {
            Err(())
        }
    }

    fn parse_struct_def(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip the `struct` keyword
        self.advance();
        if let Some((_, name)) = self.parse_ident() {
            let generics = self.parse_maybe_generics_definition()?;

            if !self.eat(TokenType::OpenCurly) {
                return Err(());
            }

            fn parse_param_with_vis(
                parser: &mut Parser,
            ) -> Result<Option<(Visibility, String, Ty)>, ()> {
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
                generics,
                fields: fields.into_boxed_slice(),
            }))
        } else {
            Err(())
        }
    }

    fn parse_trait_def(&mut self, visibility: Option<Visibility>) -> Result<ItemKind, ()> {
        // skip the `trait` keyword
        self.advance();
        if let Some((_, name)) = self.parse_ident() {
            let generics = self.parse_maybe_generics_definition()?;
            let req_sub_traits = if self.eat(TokenType::Colon) {
                let mut sub_traits = vec![];
                sub_traits.push(self.parse_ty()?);

                while self.eat_bin_op(BinOp::Add) {
                    sub_traits.push(self.parse_ty()?);
                }
                sub_traits.into_boxed_slice()
            } else {
                Box::new([])
            };

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
                generics,
                req_sub_traits,
                methods: methods.into_boxed_slice(),
            }))
        } else {
            Err(())
        }
    }

    fn parse_impl_block(&mut self) -> Result<ItemKind, ()> {
        // skip the `impl` keyword
        self.advance();

        let generics = self.parse_maybe_generics_definition()?;
        let ty = self.parse_ty()?;
        let (impl_trait, ty) = if self.eat_kw(Keyword::For) {
            let tait = self.parse_ty()?;
            (Some(ty), tait)
        } else {
            (None, ty)
        };

        if !self.eat(TokenType::OpenCurly) {
            return Err(());
        }

        let mut methods = vec![];
        let mut visibility = self.parse_visibility();
        // collect all functions inside the impl block
        while self.check_kw(Keyword::Fn) {
            let function = self.parse_function(visibility.take())?;
            methods.push(function);
            visibility = self.parse_visibility();
        }
        // check for invalid trailing visibility modifier
        if visibility.is_some() {
            return Err(());
        }

        if !self.eat(TokenType::ClosedCurly) {
            return Err(());
        }

        Ok(ItemKind::StructImpl(AdtImpl {
            ty,
            impl_trait,
            generics,
            methods: methods.into_boxed_slice(),
        }))
    }

    fn parse_glob(&mut self) -> Result<ItemKind, ()> {
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
                    Keyword::Static => self.parse_static(visibility),
                    Keyword::Const => {
                        if self.token_stream.look_ahead(1, |x| x.to_type() == TokenType::Ident) {
                            self.parse_const(visibility)
                        } else {
                            // FIXME: parse function attrs and then the function itself
                            println!("don't parse const!");
                            Err(())
                        }
                    }
                    Keyword::Rt => Err(()), // FIXME: ?
                    Keyword::Fn => self.parse_function(visibility),
                    Keyword::Enum => Err(()),
                    Keyword::Struct => self.parse_struct_def(visibility),
                    Keyword::Mod => Err(()),
                    Keyword::Impl => self.parse_impl_block(),
                    Keyword::Async => Err(()),
                    Keyword::Unsafe => Err(()),
                    Keyword::Extern => Err(()),
                    Keyword::Trait => self.parse_trait_def(visibility),
                    Keyword::Type => Err(()),
                    _ => Err(()), // FIXME: error
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
            Token::Comment(_, _) => {} // FIXME: this is currently filtered in the tokenstream
            _ => {}
        }
        Err(())
    }

    fn parse_primary(&mut self) -> Result<AstNode, ()> {
        println!("curr: {:?}", self.curr);
        match &self.curr {
            Token::Ident(_, content) => {
                if self
                    .token_stream
                    .look_ahead(1, |token| token.to_type() == TokenType::OpenParen)
                {
                    self.parse_call() // FIXME: handle errors properly!
                                      /*} else if self.token_stream.look_ahead(1, |token| token.to_type() == TokenType::Dot) {
                                      // FIXME: parse field access/struct method call
                                       */
                } else if self
                    .token_stream
                    .look_ahead(1, |token| token.to_type() == TokenType::OpenCurly)
                {
                    self.parse_struct_constructor()
                } else {
                    // FIXME: handle the rest!
                    let content = content.clone();
                    self.advance();
                    Ok(AstNode::Ident(content))
                }
            }
            //#!Token::Keyword(_, _) => {}
            // Token::StrLit(_, _) => {}
            Token::NumLit(_, _) => self.parse_number_expr(),
            Token::OpenParen(_) => self.parse_paren_expr(),
            Token::OpenBracket(_) => self.parse_array_constructor(),
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
            // Token::Comment(_, _) => Ok(None), // FIXME: this is currently filtered in the tokenstream
            Token::EOF(_) => Err(()),
            _ => Err(()),
        }
    }

    /// parses an expression for example in the body of a method
    fn parse_expr(&mut self) -> Result<AstNode, ()> {
        if self.check(TokenType::OpenCurly) {
            return self
                .parse_block_no_attr()
                .map(|block| AstNode::Block(block));
        }
        self.parse_bin_op()
    }

    fn parse_item(&mut self) -> Result<ItemKind, ()> {
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
            // Token::Comment(_, _) => Ok(None), // FIXME: this is currently filtered in the tokenstream
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

    fn eat_bin_op(&mut self, bin_op: BinOp) -> bool {
        if let Token::BinOp(_, curr_bin_op) = self.curr {
            if bin_op == curr_bin_op {
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

    // FIXME: this is currently filtered in the tokenstream
    fn skip_comments(&mut self) {
        while self.curr.to_type() == TokenType::Comment {
            self.advance();
        }
    }
}

const EOF_TOKEN: Token = Token::EOF(FixedTokenSpan::<1>::NONE);

#[cfg(test)]
fn test_file<F: FnOnce(Vec<Token>, Crate) -> bool>(path: &str, assumed: F) -> bool {
    let file = fs::read_to_string(path).unwrap();
    let lexed = lexer::lex(file).unwrap();
    let mut token_stream = TokenStream::new(lexed.clone());
    let mut parser = Parser::new(token_stream);
    let krate = parser.parse_crate().unwrap();
    assumed(lexed, krate)
}

#[test]
fn test_func() {
    assert!(test_file("tests/func.tf", |tokens, krate| tokens.len()
        == 33
        && krate.items.len() == 2));
}

#[test]
fn test_static() {
    assert!(test_file("tests/static.tf", |tokens, krate| tokens.len()
        == 24
        && krate.items.len() == 2));
}

#[test]
fn test_struct() {
    assert!(test_file("tests/struct.tf", |tokens, krate| tokens.len()
        == 19
        && krate.items.len() == 2));
}

#[test]
fn test_trait() {
    assert!(test_file("tests/trait.tf", |tokens, krate| tokens.len()
        == 29
        && krate.items.len() == 2));
}

#[test]
fn test_impl() {
    assert!(test_file("tests/impl.tf", |tokens, krate| tokens.len()
        == 33
        && krate.items.len() == 4));
}

#[test]
fn test_struct_constructor() {
    assert!(test_file(
        "tests/struct_constructor.tf",
        |tokens, krate| tokens.len() == 32 && krate.items.len() == 2
    ));
}

#[test]
fn test_generics() {
    assert!(test_file("tests/generics.tf", |tokens, krate| tokens.len()
        == 93
        && krate.items.len() == 7));
}

#[test]
fn test_ref() {
    assert!(test_file("tests/ref.tf", |tokens, krate| tokens.len()
        == 74
        && krate.items.len() == 5));
}

#[test]
fn test_array() {
    assert!(test_file("tests/array.tf", |tokens, krate| tokens.len()
        == 72
        && krate.items.len() == 2));
}
