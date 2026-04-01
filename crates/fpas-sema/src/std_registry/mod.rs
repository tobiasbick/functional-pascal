//! Standard library symbols and polymorphic `BuiltinStd` checking.
//!
//! **Documentation:** `docs/pascal/std/*.md` (from the repository root) - Pascal-facing API tables per unit.
//! **Maintenance:** Registering or changing a `Std.*` symbol here must be reflected in the matching unit
//! Markdown file and in runtime/compiler/bytecode code.

mod aliases;
mod builtins;
mod loaded;

pub use aliases::register_short_aliases;
pub use builtins::check_builtin_std_call;
pub use loaded::{register_loaded_std, register_single_std_unit};

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::*;

fn p(name: &str, ty: Ty, mutable: bool) -> ParamTy {
    ParamTy {
        mutable,
        name: name.to_string(),
        ty,
    }
}

fn define_func(c: &mut Checker, q: &str, params: Vec<ParamTy>, ret: Ty) {
    c.scopes.define(
        q,
        Symbol {
            ty: Ty::Function(FunctionTy {
                type_params: Vec::new(),
                params,
                return_type: Box::new(ret),
                variadic: false,
            }),
            mutable: false,
            kind: SymbolKind::Function,
        },
    );
}

fn define_func_variadic(c: &mut Checker, q: &str, params: Vec<ParamTy>, ret: Ty) {
    c.scopes.define(
        q,
        Symbol {
            ty: Ty::Function(FunctionTy {
                type_params: Vec::new(),
                params,
                return_type: Box::new(ret),
                variadic: true,
            }),
            mutable: false,
            kind: SymbolKind::Function,
        },
    );
}

fn define_proc_variadic(c: &mut Checker, q: &str) {
    c.scopes.define(
        q,
        Symbol {
            ty: Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params: vec![],
                variadic: true,
            }),
            mutable: false,
            kind: SymbolKind::Procedure,
        },
    );
}

fn define_proc(c: &mut Checker, q: &str, params: Vec<ParamTy>) {
    c.scopes.define(
        q,
        Symbol {
            ty: Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params,
                variadic: false,
            }),
            mutable: false,
            kind: SymbolKind::Procedure,
        },
    );
}

fn define_const(c: &mut Checker, q: &str, ty: Ty) {
    c.scopes.define(
        q,
        Symbol {
            ty,
            mutable: false,
            kind: SymbolKind::Const,
        },
    );
}

fn define_builtin_std(c: &mut Checker, q: &str, placeholder: Ty) {
    c.scopes.define(
        q,
        Symbol {
            ty: placeholder,
            mutable: false,
            kind: SymbolKind::BuiltinStd,
        },
    );
}
