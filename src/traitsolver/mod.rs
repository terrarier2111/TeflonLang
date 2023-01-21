use crate::parser::ast::Ty;
use std::collections::HashMap;

// FIXME: maybe helpful: https://rustc-dev-guide.rust-lang.org/traits/resolution.html
// https://smallcultfollowing.com/babysteps/blog/2017/01/26/lowering-rust-traits-to-logic/
// https://github.com/rust-lang/chalk
// https://rust-lang.github.io/chalk/book/#chalk-works-by-converting-rust-goals-into-logical-inference-rules
// https://www.youtube.com/channel/UCqeVI8YTNP2K3fn6N-ZHP0g/featured
// https://rust-lang.github.io/chalk/book/engine/slg.html


// https://smallcultfollowing.com/babysteps/blog/2017/03/25/unification-in-chalk-part-1/

// think of it as in:
// Vec<T>
// `T` is a type variable
// and `Vec` is an application
// so we would have an application `Vec T` here.
// another example:
// i32
// here `i32` is an application with 0 arguments

pub struct TraitManager {
    known_impls: HashMap<Ty, Vec<Vec<Ty>>>, // known impls maps a ty to a list of lists of obligations
}

impl TraitManager {
    fn insert_impl(&mut self, tait: &Ty, obligations: Vec<Ty>) {
        // FIXME: this system is incomplete, how should we handle things like `impl<T: Clone> Trait for Test<T> {`?
    }
}
