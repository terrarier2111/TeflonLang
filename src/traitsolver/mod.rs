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
        // FIXME: trait Trait<T: Bound1> {}
        // FIXME: for: impl<K: Bound2> Trait<K> for Ty {}
        // FIXME: check if Bound2 is at least as strict as Bound1
        // FIXME: more generally: (check if bounds for generics passed to Tait's generics are at least as strict as the bounds of Tait's generics themselves)

        // FIXME: this system is incomplete, how should we handle things like `impl<T: Clone> Trait for Test<T> {`?
        self.impl_constraints.entry(tait.kind.simple_ty_name()).or_insert_with(|| TraitEntry { tait: tait.clone(), goals: vec![] }).goals.push((obligations, ctx));
    }

    pub fn has_impl(&self, ty: &Ty, tait: &Ty) -> bool {
        if let Some(ty_val) = self.impl_constraints.get(&tait.to_string()) {
            'outer: for (imp, ctx) in &ty_val.goals {
                match &imp {
                    // case: impl<T: X + Y + Z> Trait for T {}
                    GoalTarget::Obligation { constraints } => {
                        for constraint in constraints {
                            if !self.has_impl(ty, constraint) {
                                continue 'outer;
                            }
                        }
                        return true;
                    },
                    // case: impl<T: X + Y + Z, K> Trait for Ty<T, K> {}
                    GoalTarget::Val { ty: val_ty, generics } => {
                        assert!(val_ty.kind.get_owned().unwrap().generics.len() == generics.len());
                        // if parent types don't match then don't even check generics
                        if val_ty.kind.to_string() != ty.kind.to_string() {
                            return false;
                        }
                        if !self.check_generics(ty, ctx) {
                            continue 'outer;
                        }
                        return true;
                    },
                }
            }
        }
        false
    }

    fn check_generics(&self, ty: &Ty, ctx: &HashMap<String, Vec<Ty>>) -> bool {
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
                return false;
            }
        }
        true
    }
}
