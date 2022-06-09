use crate::parser::attrs::{Constness, Publicity};

pub enum AstNode {
    Number(NumberType),
    Ident(String),
    BinaryExpr(BinaryExprNode),
    FunctionDef(FunctionNode),
    CallExpr(CallExprNode),
}

pub struct BinaryExprNode {
    lhs: AstNode,
    rhs: AstNode,
    op: BinaryExpr,
}

pub enum BinaryExpr {
    And,
    Or,
}

pub struct CallExprNode {
    pub(crate) callee: String,
    pub(crate) args: Vec<AstNode>,
}

pub struct FunctionHeaderNode {
    pub name: String,
    pub modifiers: FunctionModifiers,
    pub args: Vec<(String, String)>, // type, name
}

pub struct FunctionModifiers {
    constness: Constness,
    // extern_abi: Option<String>,
    publicity: Publicity,
    // is_async: bool,
}

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
