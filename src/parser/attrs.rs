#[derive(Debug, Copy, Clone)]
pub enum Constness {
    Undefined,
    Const,
    Rt, // runtime
}

#[derive(Debug, Copy, Clone)]
pub enum Mutability {
    Mut,
    Immut,
}

#[derive(Debug, Copy, Clone)]
pub enum Publicity {
    Public,
    Crate,
    Private,
}

#[derive(Debug, Copy, Clone)]
pub enum Unsafety {
    Unsafe,
    Safe,
}
