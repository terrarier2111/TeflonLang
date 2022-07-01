use crate::diagnostics::span::Span;
use crate::lexer::token::BinOp;
use crate::parser::attrs::{Constness, Mutability, Visibility};

// FIXME: interesting: https://en.wikipedia.org/wiki/Terminal_and_nonterminal_symbols

#[derive(Debug, Clone)]
pub struct Crate {
    pub(crate) items: Box<[ItemKind]>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Number(NumberType),
    Ident(String),
    BinaryExpr(Box<BinaryExprNode>),
    CallExpr(CallExprNode),
    Block(Block),
    StructConstructor(StructConstructor), // FIXME: should this be renamed to StructInit?
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub(crate) val: AstNode,
    // pub(crate) span: Span, // FIXME: somehow retrieve(and keep) span information
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Item(ItemKind),
    LocalAssign(LocalAssign),
    Expr(AstNode),
    Semi(AstNode), // FIXME: for now we put block exprs into Semi stmts, but change this ASAP
    Empty,
}

#[derive(Debug, Clone)]
pub enum ItemKind {
    StaticVal(Box<StaticValNode>),
    ConstVal(Box<ConstValNode>),
    FunctionDef(Box<FunctionNode>),
    StructDef(StructDef),
    TraitDef(TraitDef),
    StructImpl(StructImpl),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub(crate) modifiers: BlockModifiers,
    pub(crate) stmts: Box<[StmtKind] /*[Stmt]*/>, // FIXME: switch to stmts
}

#[derive(Debug, Clone)]
pub struct BlockModifiers {}

// every expression consists of some(or none) statements
// and at most one expression at its end

#[derive(Debug, Clone)]
pub struct BinaryExprNode {
    pub(crate) lhs: AstNode,
    pub(crate) rhs: AstNode,
    pub(crate) op: BinOp,
}

#[derive(Debug, Clone)]
pub struct CallExprNode {
    pub(crate) callee: String,
    pub(crate) args: Box<[AstNode]>,
}

#[derive(Debug, Clone)]
pub struct StaticValNode {
    pub(crate) ty: Ty,
    // name is contained within val as its lhs field
    pub(crate) val: AstNode,
    pub(crate) visibility: Option<Visibility>,
    pub(crate) mutability: Option<Mutability>,
}

impl StaticValNode {
    pub fn left(&self) -> &String {
        if let AstNode::Ident(lhs) = &self.val {
            lhs
        } else {
            panic!("The lhs node of the assignment was {:?} and not the name of the variable it was assigned to!", self.val)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstValNode {
    pub(crate) ty: Ty,
    // name is contained within val as its lhs field
    pub(crate) val: AstNode,
    pub(crate) visibility: Option<Visibility>,
}

impl ConstValNode {
    pub fn left(&self) -> &String {
        if let AstNode::Ident(lhs) = &self.val {
            lhs
        } else {
            panic!("The lhs node of the assignment was {:?} and not the name of the variable it was assigned to!", self.val)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionModifiers {
    pub(crate) constness: Constness,
    // extern_abi: Option<String>,
    pub(crate) visibility: Visibility,
    // is_async: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub(crate) modifiers: FunctionModifiers,
    pub(crate) header: FunctionHeader,
    pub(crate) body: Block,
}

#[derive(Debug, Clone)]
pub struct FunctionHeader {
    pub(crate) name: String,
    pub(crate) args: Box<[(String, Ty)]>, // name, type
    pub(crate) ret: Option<Ty>,           // type
}

#[derive(Debug, Clone)]
pub enum LocalAssign {
    Assign(LAssign),
    DecAssign(LDecAssign),
}

#[derive(Debug, Clone)]
pub struct LAssign {
    pub(crate) name: String,
    pub(crate) val: AstNode,
}

#[derive(Debug, Clone)]
pub struct LDecAssign {
    // LocalDeclareAssignment
    pub(crate) ty: Option<Ty>,
    pub(crate) val: LAssign,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) fields: Box<[StructFieldDef]>,
}

#[derive(Debug, Clone)]
pub struct StructFieldDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub(crate) visibility: Visibility,
    pub(crate) name: String,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) req_sub_traits: Box<[Ty]>,
    pub(crate) methods: Box<[FunctionHeader]>,
}

#[derive(Debug, Clone)]
pub struct StructImpl {
    pub(crate) ty: TyOrGeneric,
    pub(crate) impl_trait: Option<Ty>,
    pub(crate) generics: Box<[Generic]>,
    pub(crate) methods: Box<[ItemKind]>,
}

#[derive(Debug, Clone)]
pub struct StructConstructor {
    pub(crate) name: String,
    pub(crate) fields: Box<[(String, AstNode)]>,
}

#[derive(Debug, Clone)]
pub enum TyOrGeneric {
    Ty(Ty),
    Generic(String),
}

#[derive(Debug, Clone)]
pub enum Generic {
    Constant(GenericConstant),
    Type(GenericType),
    Lifetime(GenericLifetime),
}

#[derive(Debug, Clone)]
pub struct GenericType {
    pub(crate) name: String,
    pub(crate) required_traits: Box<[Ty]>,
}

#[derive(Debug, Clone)]
pub struct GenericLifetime {
    pub(crate) lt: Lifetime,
    // pub(crate) constraints: Box<[Lifetime]>, // FIXME: implement this!
}

#[derive(Debug, Clone)]
pub enum Lifetime {
    Custom(String),
    Static,
    Inferred,
}

#[derive(Debug, Clone)]
pub struct GenericConstant {
    pub(crate) name: String,
    pub(crate) ty: Ty,
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub(crate) name: String,
    pub(crate) generics: Box<[TyOrConstVal]>,
}

#[derive(Debug, Clone)]
pub enum TyOrConstVal {
    Ty(Ty),
    ConstVal(AstNode),
}

#[derive(Debug, Copy, Clone)]
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
