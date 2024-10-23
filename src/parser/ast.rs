use std::hash::Hash;

use crate::diagnostics::span::Span;
use crate::lexer::token::BinOp;
use crate::parser::attrs::{Constness, Mutability, Visibility};
use crate::tyck::{Adt, DEFAULT_PATH, EMPTY, Environment, TyCtx};

// FIXME: interesting: https://en.wikipedia.org/wiki/Terminal_and_nonterminal_symbols

#[derive(Debug, Clone, PartialEq)]
pub struct Crate {
    pub(crate) items: Box<[ItemKind]>,
}

impl Crate {

    pub fn build_ctx(&self) -> TyCtx {
        let mut ret = TyCtx {
            env: Environment::new(),
        };

        // early resolution
        for item in &*self.items {
            ret.insert_item_glob(item);
        }

        ret
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    Number(NumberType),
    Ident(String),
    BinaryExpr(Box<BinaryExprNode>),
    CallExpr(CallExprNode),
    Block(Block),
    StructConstructor(StructConstructor), // FIXME: should this be renamed to StructInit?
    ArrayInst(ArrayInst),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub(crate) val: AstNode,
    // pub(crate) span: Span, // FIXME: somehow retrieve(and keep) span information
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Item(ItemKind),
    LocalAssign(LocalAssign),
    Expr(AstNode),
    Semi(AstNode), // FIXME: for now we put block exprs into Semi stmts, but change this ASAP
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    StaticVal(Box<StaticValNode>),
    ConstVal(Box<ConstValNode>),
    FunctionDef(Box<FunctionNode>),
    StructDef(StructDef),
    TraitDef(TraitDef),
    StructImpl(AdtImpl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub(crate) modifiers: BlockModifiers,
    pub(crate) stmts: Box<[StmtKind] /*[Stmt]*/>, // FIXME: switch to stmts
    // FIXME: th last thingy should be either empty or an AstNode
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockModifiers {}

// every expression consists of some(or none) statements
// and at most one expression at its end

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryExprNode {
    pub(crate) lhs: AstNode,
    pub(crate) rhs: AstNode,
    pub(crate) op: BinOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallExprNode {
    pub(crate) callee: String,
    pub(crate) args: Box<[AstNode]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticValNode {
    pub(crate) ty: Ty,
    // name is contained within val as its lhs field
    pub(crate) val: AstNode,
    pub(crate) visibility: Option<Visibility>,
    pub(crate) mutability: Option<Mutability>,
}

impl StaticValNode {
    pub fn left(&self) -> &String {
        if let AstNode::BinaryExpr(bin) = &self.val {
            if let AstNode::Ident(lhs) = &bin.lhs {
                lhs
            } else {
                panic!("The lhs node of the assignment was {:?} and not the name of the variable it was assigned to!", self.val)
            }
        } else {
            panic!("The node of the assignment was {:?} and not a binary expression!", self.val)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstValNode {
    pub(crate) ty: Ty,
    // name is contained within val as its lhs field
    pub(crate) val: AstNode,
    pub(crate) visibility: Option<Visibility>,
}

impl ConstValNode {
    pub fn left(&self) -> &String {
        if let AstNode::BinaryExpr(bin) = &self.val {
            if let AstNode::Ident(lhs) = &bin.lhs {
                lhs
            } else {
                panic!("The lhs node of the assignment was {:?} and not the name of the variable it was assigned to!", self.val)
            }
        } else {
            panic!("The node of the assignment was {:?} and not a binary expression!", self.val)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FunctionModifiers {
    pub(crate) constness: Constness,
    // extern_abi: Option<String>,
    pub(crate) visibility: Visibility,
    // is_async: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionNode {
    pub(crate) modifiers: FunctionModifiers,
    pub(crate) header: FunctionHeader,
    pub(crate) body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHeader {
    pub(crate) name: String,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) args: Box<[(String, Ty)]>, // name, type
    pub(crate) ret: Option<Ty>,           // type
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalAssign {
    Assign(LAssign),
    DecAssign(LDecAssign),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LAssign {
    pub(crate) name: String,
    pub(crate) val: AstNode,
}

// LocalDeclareAssignment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LDecAssign {
    pub(crate) mutability: Option<Mutability>,
    pub(crate) ty: Option<Ty>,
    pub(crate) val: LAssign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) fields: Box<[StructFieldDef]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) req_sub_traits: Box<[Ty]>, // this may not be generic
    pub(crate) methods: Box<[FunctionHeader]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtImpl {
    pub(crate) ty: Ty,
    pub(crate) impl_trait: Option<Ty>, // this may not be generic
    pub(crate) generics: Box<[Generic]>,
    pub(crate) methods: Box<[ItemKind]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructConstructor {
    pub(crate) name: String,
    pub(crate) fields: Box<[(String, AstNode)]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Generic {
    Constant(GenericConstant),
    Type(GenericType),
    Lifetime(GenericLifetime),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericType {
    pub(crate) name: String,
    pub(crate) required_traits: Box<[Ty]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericLifetime {
    pub(crate) lt: Lifetime,
    // pub(crate) constraints: Box<[Lifetime]>, // FIXME: implement this!
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifetime {
    Custom(String),
    Static,
    Inferred,
}

impl Lifetime {

    fn to_string(&self) -> &str {
        match self {
            Lifetime::Custom(val) => &val,
            Lifetime::Static => "static",
            Lifetime::Inferred => "_",
        }
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericConstant {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ty {
    pub(crate) kind: TyKind,
    // FIXME: add span
}

impl Hash for Ty {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Ty {

    pub fn to_string(&self) -> String {
        self.kind.to_string()
    }

    /*pub fn to_simple_string(&self) -> String {
        self.kind.to_simple_string()
    }*/

}

// FIXME: introduce and use TyKind in the future (use smth similar for generics)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyKind {
    Ref(Box<RefTy>),
    Array(Box<ArrayTy>),
    // Ptr, // TODO
    Owned(Box<OwnedTy>),
}

impl TyKind {

    pub fn get_generics(&self) -> &[TyOrConstVal] {
        match self {
            Self::Ref(rf) => rf.ty.kind.get_generics(),
            Self::Array(array) => array.ty.kind.get_generics(),
            Self::Owned(ty) => &ty.generics,
        }
    }

    pub fn simple_ty_name(&self) -> String {
        match self {
            Self::Ref(rf) => format!("&{}{}{}", if let Some(lt) = rf.lt.as_ref() { if lt == &Lifetime::Static { " 'static" } else { " '_" } } else { "" }, if rf.mutability == Mutability::Mut { " mut" } else { "" }, rf.ty.kind.simple_ty_name()),
            Self::Array(array) => {
                let mut raw = "[".to_string();
                raw.push_str(&array.ty.kind.simple_ty_name());
                if let Some(amount) = &array.amount {
                    raw.push_str("; ");
                    // FIXME: print amount!
                }
                raw.push(']');
                raw
            },
            Self::Owned(ty) => ty.name.clone(),
        }
    }

    pub fn get_owned(&self) -> Option<&OwnedTy> {
        match self {
            TyKind::Ref(_) => None,
            TyKind::Array(_) => None,
            TyKind::Owned(ty) => Some(&ty),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TyKind::Ref(ref_ty) => ref_ty.to_string(),
            TyKind::Array(array_ty) => array_ty.to_string(),
            TyKind::Owned(owned_ty) => owned_ty.to_string(),
        }
    }

    /*pub fn to_simple_string(&self) -> String {
        match self {
            TyKind::Ref(ref_ty) => ref_ty.to_simple_string(),
            TyKind::Array(array_ty) => array_ty.to_simple_string(),
            TyKind::Owned(owned_ty) => owned_ty.to_simple_string(),
        }
    }*/

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefTy {
    pub(crate) lt: Option<Lifetime>,
    pub(crate) mutability: Mutability,
    pub(crate) ty: Box<Ty>,
}

impl RefTy {

    fn to_string(&self) -> String {
        format!("&{}{}{}", if let Some(lt) = &self.lt { format!(" '{}", lt.to_string()) } else { String::new() }, if self.mutability == Mutability::Mut { "mut " } else { "" }, self.ty.to_string())
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedTy {
    pub(crate) name: String,
    pub(crate) generics: Box<[TyOrConstVal]>,
}

impl OwnedTy {

    fn to_string(&self) -> String {
        let mut raw = self.name.clone();
        if !self.generics.is_empty() {
            raw.push('<');
            for val in &self.generics {
                raw.push_str(&val.to_string());
                raw.push_str(", ");
            }
            raw.pop();
            raw.pop();
            raw.push('>');
        }
        raw
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayTy {
    pub(crate) ty: Ty,
    pub(crate) amount: Option<AstNode>,
}

impl ArrayTy {

    fn to_string(&self) -> String {
        let mut raw = "[".to_string();
        raw.push_str(&self.ty.to_string());
        if let Some(amount) = &self.amount {
            raw.push_str("; ");
            // FIXME: print amount!
        }
        raw.push(']');
        raw
    }

}

/// This represents an array instantiation like: [6, 4, 2, 8, 3, 9] or [6; 20]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayInst {
    List(ArrayInstList),
    Short(Box<ArrayInstShort>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayInstList {
    pub(crate) vals: Box<[AstNode]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayInstShort {
    pub(crate) val: AstNode,
    pub(crate) amount: AstNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayIndexing {
    pub(crate) array: Path,
    pub(crate) idx_val: AstNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    // FIXME: design this properly!
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TyOrConstVal {
    Ty(Ty),
    ConstVal(AstNode),
}

impl TyOrConstVal {

    pub fn to_string(&self) -> String {
        match self {
            TyOrConstVal::Ty(ty) => ty.to_string(),
            TyOrConstVal::ConstVal(ast_node) => todo!(),
        }
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NumberType {
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

/// FIXME: currently we assume equality for floats
impl Eq for NumberType {}
