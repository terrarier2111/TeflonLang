use crate::parser::keyword::Keyword::{
    Else, Enum, Fn, For, If, Impl, In, Let, Loop, Match, Mod, Mut, Pub, Rt, SelfLower, SelfUpper,
    Static, Struct, Trait, Type, While,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Keyword {
    Pub,
    Static,
    Const,
    Rt, // runtime modifier (aka non-const)
    Let,
    Fn,
    Mut,
    Enum,
    Struct,
    Mod,
    SelfUpper, // Self
    SelfLower, // self
    Impl,
    If,
    Else,
    Match,
    For,
    While,
    Loop,
    In,
    Async,
    Unsafe,
    Extern,
    Trait,
    Type,
    // FIXME: there should be a couple of other keywords missing here
}

impl Keyword {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "pub" => Some(Pub),
            "static" => Some(Static),
            "runtime" => Some(Rt),
            "let" => Some(Let),
            "fn" => Some(Fn),
            "mut" => Some(Mut),
            "enum" => Some(Enum),
            "struct" => Some(Struct),
            "mod" => Some(Mod),
            "Self" => Some(SelfUpper),
            "self" => Some(SelfLower),
            "impl" => Some(Impl),
            "if" => Some(If),
            "else" => Some(Else),
            "match" => Some(Match),
            "for" => Some(For),
            "while" => Some(While),
            "loop" => Some(Loop),
            "in" => Some(In),
            "trait" => Some(Trait),
            "type" => Some(Type),
            _ => None,
        }
    }
}
