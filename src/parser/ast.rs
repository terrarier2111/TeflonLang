use crate::lexer::token::BinOp;
use crate::parser::attrs::{Constness, Visibility};

#[derive(Debug, Clone)]
pub enum AstNode {
    Number(NumberType),
    Ident(String),
    BinaryExpr(Box<BinaryExprNode>),
    FunctionDef(Box<FunctionNode>),
    CallExpr(CallExprNode),
}

#[derive(Debug, Clone)]
pub struct BinaryExprNode {
    pub(crate) lhs: AstNode,
    pub(crate) rhs: AstNode,
    pub(crate) op: BinOp,
}

#[derive(Debug, Clone)]
pub struct CallExprNode {
    pub(crate) callee: String,
    pub(crate) args: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct FunctionHeaderNode {
    pub name: String,
    pub modifiers: FunctionModifiers,
    pub args: Vec<(String, String)>, // type, name
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionModifiers {
    constness: Constness,
    // extern_abi: Option<String>,
    visibility: Visibility,
    // is_async: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub(crate) header: FunctionHeaderNode,
    pub(crate) body: AstNode,
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
