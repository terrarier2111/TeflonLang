use crate::parser::ast::{AstNode, Ty, TyOrConstVal};
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
    /// impl trait for ty
    /// -> map<simple trait name, (trait, list[any{(ty, generic obligations), obligations}])>
    impl_constraints: HashMap<String, TraitEntry>,
    /// map<trait, list[ty]>
    used_impls: HashMap<Ty, Vec<Ty>>,
}

struct TraitEntry {
    tait: Ty,
    goals: Vec<(GoalTarget, HashMap<String, Vec<Ty>>)>,
}

pub enum GenericGoalTarget {
    Val {
        ty: Ty,
        generics: Vec<GenericGoalTarget>,
    },
    Const {
        val: AstNode,
    },
    Obligation {
        constraints: Vec<Ty>,
    },
}

pub enum GoalTarget {
    Val {
        ty: Ty,
        generics: Vec<GoalTarget>,
    },
    Obligation {
        constraints: Vec<Ty>,
    },
}

impl TraitManager {
    pub fn insert_impl(&mut self, tait: &Ty, obligations: GoalTarget, ctx: HashMap<String, Vec<Ty>>) {
        // FIXME: this system is incomplete, how should we handle things like `impl<T: Clone> Trait for Test<T> {`?
        self.impl_constraints.entry(tait.to_simple_string()).or_insert_with(|| TraitEntry { tait: tait.clone(), goals: vec![] }).goals.push((obligations, ctx));
    }

    pub fn has_impl(&self, ty: &Ty, tait: &Ty) -> bool {
        if let Some(ty_val) = self.impl_constraints.get(&tait.to_simple_string()) {
            'outer: for (imp, ctx) in &ty_val.goals {
                match &imp {
                    GoalTarget::Obligation { constraints } => {
                        for constraint in constraints {
                            if !self.has_impl(ty, constraint) {
                                continue 'outer;
                            }
                        }
                        return true;
                    },
                    GoalTarget::Val { ty, generics } => {
                        assert!(ty.kind.get_owned().unwrap().generics.len() == generics.len());
                        for generic in &ty.kind.get_owned().unwrap().generics {
                            if !match generic {
                                TyOrConstVal::ConstVal(val) => todo!(),
                                // FIXME: we need to be able to check this ty's generics as well!
                                TyOrConstVal::Ty(generic_ty) => {
                                    if let Some(val) = generic_ty.kind.get_owned() {
                                        if let Some(constraints) = ctx.get(&val.name) {
                                            assert!(val.generics.is_empty());
                                            
                                        }
                                    }
                                },
                            } {
                                continue 'outer;
                            }
                        }
                        return true;
                    },
                }
            }
        }
        false
    }
}
