// https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system
// https://en.wikipedia.org/wiki/Attribute_grammar

// https://www.youtube.com/watch?v=-TQVAKby6oI
// https://www.youtube.com/watch?v=9gbZ_DlG7CY

// https://cs.stackexchange.com/questions/148/type-checking-algorithms

// https://www.youtube.com/watch?v=frM7GhBERAs | nice
// https://github.com/kritzcreek/infer_workshop/

// https://rustc-dev-guide.rust-lang.org/type-inference.html#trying-equality
// https://rustc-dev-guide.rust-lang.org/type-checking.html
// https://rustc-dev-guide.rust-lang.org/traits/resolution.html

// https://www.cs.cornell.edu/andru/papers/gallifrey-types/gallifrey-types.pdf

// MAYBE: https://gist.github.com/aradarbel10/d26ce198863b537d902e55f8ec9f3b37

// bruijn indices

// https://www.youtube.com/watch?v=utyBNDj7s2w
// https://www.youtube.com/watch?v=uJHD2xyv7xo
// https://www.youtube.com/watch?v=KWsjMWqAXlg

// https://github.com/audulus/lyte

mod infer;

use crate::parser::attrs::{Mutability, Visibility};
use std::collections::HashMap;
use std::string::ToString;
use lazy_static::lazy_static;
use crate::parser::ast;
use crate::parser::ast::{ArrayInst, AstNode, FunctionNode, StmtKind, StructDef, AdtImpl, TyKind, TyOrConstVal};

pub const DEFAULT_PATH: &str = ""; // TODO: get rid of this once paths are properly implemented!

pub struct TyCtx {
    pub(crate) env: Environment,
}

lazy_static! {
    pub static ref EMPTY: TyCtx = TyCtx {
        env: Environment {
            scopes: vec![],
            adts_by_path: HashMap::new(),
            adt_impls_by_path: HashMap::new(),
            funcs_by_path: HashMap::new(),
        },
    };
}

impl TyCtx {

    pub fn resolve_ty(&self, ast_node: &AstNode) -> Option<Ty> {
        match ast_node {
            AstNode::Number(_) => Some(Ty::Primitive(PrimitiveTy::UnsizedInt)),
            AstNode::Ident(ident) => self.env.resolve_var(ident),
            AstNode::BinaryExpr(expr) => {
                // FIXME: support different return types (as in different from the base type)
                self.resolve_ty(&expr.lhs)
            }
            AstNode::CallExpr(call) => {
                self.env.resolve_func(&call.callee).map(|x| x.header.ret.clone().map(|ty| Ty::from_ast_ty(ty.kind, None))).flatten()
            }
            AstNode::Block(block) => {
                if let Some(last) = block.stmts.last() {
                    if let StmtKind::Expr(expr) = last {
                        self.resolve_ty(expr)
                    } else {
                        Some(Ty::Empty)
                    }
                } else {
                    Some(Ty::Empty)
                }
            }
            AstNode::StructConstructor(constructor) => {
                let ret = self.env.resolve_adt(&DEFAULT_PATH.to_string(), &constructor.name).map(|adt| &adt.1).cloned();
                println!("struct constr: {:?}", ret);
                ret
            },
            AstNode::ArrayInst(array) => {
                match array {
                    ArrayInst::List(def) => {
                        for def in &*def.vals {
                            if let Some(val) = self.resolve_ty(def) {
                                return Some(Ty::Array(ArrayTy {
                                    elem_ty: Box::new(val),
                                }));
                            }
                        }
                        None
                    }
                    ArrayInst::Short(def) => {
                        self.resolve_ty(&def.val).map(|x| Ty::Array(ArrayTy {
                            elem_ty: Box::new(x),
                        }))
                    }
                }
            }
        }
    }

    pub fn resolve_named_ty(&self, path: &String, name: &String) -> Option<&Ty> {
        self.env.adts_by_path.get(path).map(|x| x.get(name).map(|x| &x.1)).flatten()
    }

}

pub struct Environment {
    scopes: Vec<Scope>, // this is a stack of scopes which pushes a new scope up each time we enter a new scope and pops a scope each time we leave a scope
                                      // FIXME: try to make this more efficient by having an additional stack for each variable which defines its value in the current scope and
                                      // FIXME: a single HashMap for all scopes (except static ones)
    adts_by_path: HashMap<String, HashMap<String, (Adt, Ty)>>,
    adt_impls_by_path: HashMap<String, HashMap<String, Vec<AdtImpl>>>,
    funcs_by_path: HashMap<String, HashMap<String, FunctionNode>>,
}

pub enum Adt {
    Struct(StructDef),
    // Enum(), // TODO: support this!
}

impl Adt {

    pub fn to_scaffolding(&self) -> TyScaffolding {
        match self {
            Adt::Struct(_) => TyScaffolding::Struct,
        }
    }

}

pub enum Dest {
    Static(Ty),
    Local(Vec<Ty>),
}

#[derive(Default)]
struct Scope {
    vars: HashMap<String, Dest>,
    funcs: HashMap<String, FunctionNode>,
}

impl Environment {

    pub fn new() -> Self {
        Self {
            // The first entry in scopes is the static scope
            scopes: vec![Scope::default()],
            adts_by_path: Default::default(),
            adt_impls_by_path: Default::default(),
            funcs_by_path: Default::default(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) -> bool {
        if self.scopes.len() <= 1 {
            return false;
        }
        self.scopes.pop();
        true
    }

    pub fn resolve_var(&self, var: &String) -> Option<Ty> {
        // first try to resolve the var in the current scope and then from any other scope
        // in which we currently are from ascending order
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.vars.get(var) {
                let ret = match ty {
                    Dest::Static(ty) => Some(ty.clone()), // FIXME: don't clone this, do smth smarter instead!
                    Dest::Local(tys) => tys.last().cloned(), // FIXME: don't clone this, do smth smarter instead!
                };
                return ret;
            }
        }
        return None;
    }

    pub fn define_var(&mut self, var: String, ty: Ty) -> bool {
        let mut scope = self.scopes.last_mut().unwrap();

        match scope.vars.entry(var.clone()).or_insert_with(|| Dest::Local(vec![])) {
            Dest::Static(_) => false,
            Dest::Local(ref mut tys) => {
                tys.push(ty);
                true
            },
        }
    }

    pub fn define_static_var(&mut self, var: String, ty: Ty) -> bool {
        self.scopes.first_mut().unwrap().vars.try_insert(var, Dest::Static(ty)).is_ok()
    }

    pub fn resolve_func(&self, name: &String) -> Option<&FunctionNode> {
        for scope in self.scopes.iter().rev() {
            if let Some(func) = scope.funcs.get(name) {
                return Some(func);
            }
        }
        None
    }

    pub fn define_func(&mut self, name: String, func: FunctionNode) -> bool {
        let mut scope = self.scopes.last_mut().unwrap();

        scope.funcs.try_insert(name, func).is_ok()
    }

    pub fn define_static_func(&mut self, name: String, func: FunctionNode) -> bool {
        self.scopes.first_mut().unwrap().funcs.try_insert(name, func).is_ok()
    }

    pub fn resolve_adt(&self, path: &String, name: &String) -> Option<&(Adt, Ty)> {
        if let Some(path) = self.adts_by_path.get(path) {
            if let Some(func) = path.get(name) {
                return Some(func);
            }
        }
        None
    }

    pub fn define_adt(&mut self, path: String, name: String, adt: Adt) -> bool {
        let ty = match &adt {
            Adt::Struct(s_adt) => Ty::Struct(StructTy {
                vis: s_adt.visibility.clone(),
                name: s_adt.name.clone(),
                fields: {
                    let mut this = Vec::with_capacity(s_adt.fields.len());
                    for x in &*s_adt.fields {
                        this.push(StructField {
                            vis: x.visibility.clone(),
                            name: x.name.clone(),
                            ty: Ty::from_ast_ty(x.ty.kind.clone(), Some(adt.to_scaffolding())),
                        });
                    }
                    this.into_boxed_slice()
                },
            }),
        };
        println!("gen ty: {:?}", ty);
        self.adts_by_path.entry(path).or_insert_with(|| HashMap::new()).try_insert(name, (adt, ty)).is_ok()
    }

    pub fn resolve_impls(&self, path: &String, name: &String) -> Option<&Vec<AdtImpl>> {
        if let Some(path) = self.adt_impls_by_path.get(path) {
            if let Some(impls) = path.get(name) {
                return Some(impls);
            }
        }
        None
    }

    pub fn define_impl(&mut self, path: String, name: String, adt_impl: AdtImpl) {
        self.adt_impls_by_path.entry(path).or_insert_with(|| HashMap::new()).entry(name).or_insert_with(|| vec![]).push(adt_impl);
    }

}

#[derive(Debug, Clone)]
pub enum Ty {
    Empty,
    Enum(EnumTy),
    Struct(StructTy),
    Union(UnionTy),
    Tuple(TupleTy),
    Array(ArrayTy),
    Primitive(PrimitiveTy),
    Ref(RefTy),
    Unresolved(UnresolvedTy),
}

impl Ty {

    pub fn could_be(&self, other: &Ty) -> bool {
        todo!()
    }

    pub fn from_ast_ty(ast_ty: ast::TyKind, scaffolding: Option<TyScaffolding>) -> Self {
        if scaffolding.is_some() {
            println!("has scaffolding: {:?}", scaffolding);
        }
        match ast_ty {
            TyKind::Ref(rf) => Ty::Ref(RefTy {
                lt: rf.lt.map(|lt| Lifetime::from_ast_lt(lt)),
                mutability: rf.mutability,
                ty: Box::new(Self::from_ast_ty(rf.ty.kind, scaffolding)),
            }),
            TyKind::Array(array) => {
                Ty::Array(ArrayTy {
                    elem_ty: Box::new(Self::from_ast_ty(array.ty.kind, scaffolding)),
                    // FIXME: add len field
                })
            }
            TyKind::Owned(owned) => {
                Ty::Unresolved(UnresolvedTy {
                    name: owned.name,
                    generics: owned.generics,
                })
            }
        }
    }

}

#[derive(Copy, Clone, Debug)]
pub enum TyScaffolding {
    Struct,
    Enum,
    Union,
}

#[derive(Debug, Clone)]
pub struct EnumTy {
    pub name: String,
    pub vis: Visibility,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub ord: usize,
    pub fields: Vec<Ty>,
}

#[derive(Debug, Clone)]
pub struct StructTy {
    pub vis: Visibility,
    pub name: String,
    pub fields: Box<[StructField]>,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub vis: Visibility,
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct UnionTy {
    pub vis: Visibility,
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct TupleTy {
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct ArrayTy {
    pub elem_ty: Box<Ty>,
    // pub len: Option<usize>,
    // FIXME: how do we deal with length? do we store it in here or ignore it for now?
}

#[derive(Debug, Clone)]
pub enum PrimitiveTy {
    Bool,
    Char,
    Str,
    MachineSizedInt(MachineSizedIntTy),
    SizedInt(SizedIntTy),
    UnsizedInt, // an integer the size of which wasn't resolved yet and as such the default size is being used.
    SizedFloat(SizedFloatTy),
}

#[derive(Debug, Clone)]
pub struct MachineSizedIntTy {
    pub unsigned: bool,
}

#[derive(Debug, Clone)]
pub struct SizedIntTy {
    pub unsigned: bool,
    pub exp: usize, // the size of the ty as an exponent of 2, the bits can be calculated as f(x) = 8 * (2 << x)
}

impl SizedIntTy {
    pub fn bits(&self) -> usize {
        8 * (2 << self.exp)
    }
}

#[derive(Debug, Clone)]
pub struct SizedFloatTy {
    pub unsigned: bool,
    pub exp: usize, // the size of the ty as an exponent of 2, the bits can be calculated as f(x) = 32 * (2 << x)
}

impl SizedFloatTy {
    pub fn bits(&self) -> usize {
        32 * (2 << self.exp)
    }
}

#[derive(Debug, Clone)]
pub struct RefTy {
    pub lt: Option<Lifetime>,
    pub mutability: Mutability,
    pub ty: Box<Ty>,
}

#[derive(Debug, Clone)]
pub enum Lifetime {
    Custom(String),
    Static,
    Inferred,
}

impl Lifetime {

    fn from_ast_lt(ast_lt: ast::Lifetime) -> Self {
        match ast_lt {
            ast::Lifetime::Custom(name) => Lifetime::Custom(name),
            ast::Lifetime::Static => Lifetime::Static,
            ast::Lifetime::Inferred => Lifetime::Inferred,
        }
    }

}

#[derive(Debug, Clone)]
pub struct UnresolvedTy {
    pub name: String,
    pub generics: Box<[TyOrConstVal]>,
}
