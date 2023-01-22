#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Constness {
    Undefined,
    Const,
    Rt, // runtime
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mutability {
    Mut,
    Immut,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Crate,
    Private,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Unsafety {
    Unsafe,
    Safe,
}
