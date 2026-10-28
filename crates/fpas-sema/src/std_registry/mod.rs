//! Standard library symbols and polymorphic `BuiltinStd` checking.
//!
//! **Documentation:** `docs/pascal/std/*.md` (from the repository root) — Pascal-facing API tables per unit.
//! **Maintenance:** Registering or changing a `Std.*` symbol here must be reflected in the matching unit
//! Markdown file and in `fpas-builtins`, `fpas-runtime`, `fpas-code`, and `fpas-compiler`.

mod api;
mod builtins;
mod loaded;
mod native;
mod qualified_names;
mod receiver;

pub use api::{
    IntrinsicStdSymbol, IntrinsicStdSymbolKind, intrinsic_std_symbols, intrinsic_std_units,
};
pub use builtins::check_builtin_std_call_refs;
pub(crate) use loaded::{native_error_type, register_error_type};
pub use loaded::{register_loaded_std, register_single_std_unit};
pub use native::{
    NativeLowering, NativeOperation, NativeReceiver, native_factory, native_migration_hint,
    native_operation, native_operation_by_implementation, native_operations,
    validate_native_catalog,
};
pub(crate) use qualified_names::register_qualified_import_candidates;
pub use receiver::intrinsic_std_receiver_mode;

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::*;

fn p(name: &str, ty: Ty) -> ParamTy {
    ParamTy::value(name, ty)
}

fn define_func(c: &mut Checker, q: &str, params: Vec<ParamTy>, ret: Ty) {
    c.scopes.imports.define_metadata(
        q,
        Symbol {
            constant: None,
            ty: Ty::Function(FunctionTy {
                type_params: Vec::new(),
                params,
                return_type: Box::new(ret),
                variadic: false,
            }),
            mutable: false,
            kind: SymbolKind::Function,
            capability: crate::TransferCapability::Transferable,
        },
    );
}

fn define_proc_variadic(c: &mut Checker, q: &str) {
    c.scopes.imports.define_metadata(
        q,
        Symbol {
            constant: None,
            ty: Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params: vec![],
                variadic: true,
            }),
            mutable: false,
            kind: SymbolKind::Procedure,
            capability: crate::TransferCapability::Transferable,
        },
    );
}

fn define_proc(c: &mut Checker, q: &str, params: Vec<ParamTy>) {
    c.scopes.imports.define_metadata(
        q,
        Symbol {
            constant: None,
            ty: Ty::Procedure(ProcedureTy {
                type_params: Vec::new(),
                params,
                variadic: false,
            }),
            mutable: false,
            kind: SymbolKind::Procedure,
            capability: crate::TransferCapability::Transferable,
        },
    );
}

fn define_const(c: &mut Checker, q: &str, ty: Ty) {
    c.scopes.imports.define_metadata(
        q,
        Symbol {
            constant: None,
            ty,
            mutable: false,
            kind: SymbolKind::Const,
            capability: crate::TransferCapability::Transferable,
        },
    );
}

fn define_builtin_std(c: &mut Checker, q: &str, mut placeholder: Ty) {
    let params = builtins::parameter_names(q)
        .iter()
        .map(|name| p(name, Ty::Error))
        .collect();
    match &mut placeholder {
        Ty::Function(function) => function.params = params,
        Ty::Procedure(procedure) => procedure.params = params,
        _ => unreachable!("polymorphic operation must be callable"),
    }
    c.scopes.imports.define_metadata(
        q,
        Symbol {
            constant: None,
            ty: placeholder,
            mutable: false,
            kind: SymbolKind::BuiltinStd,
            capability: crate::TransferCapability::Transferable,
        },
    );
}
