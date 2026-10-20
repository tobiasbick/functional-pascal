//! Shared std unit type registration helpers.
//!
//! **Documentation:** `docs/pascal/language/types/README.md`

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use std::sync::Arc;

/// Register a simple enum type and expose each variant as a qualified enum member.
pub(super) fn register_enum_type(
    checker: &mut Checker,
    qualified_name: &str,
    variants: &[&str],
) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .enumerate()
        .map(|(index, variant)| EnumVariantTy {
            message: None,
            name: (*variant).to_string(),
            fields: vec![],
            backing_value: i64::try_from(index).ok(),
        })
        .collect();
    let enum_ty = Ty::Enum(Arc::new(EnumTy {
        is_error: false,
        name: qualified_name.into(),
        type_params: Vec::new(),
        type_args: Vec::new(),
        variants: variants.clone(),
    }));
    checker.scopes.imports.define_metadata(
        qualified_name,
        Symbol {
            constant: None,
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );

    for variant in &variants {
        let qualified_member = format!("{qualified_name}.{}", variant.name);
        checker.scopes.imports.define_metadata(
            &qualified_member,
            Symbol {
                constant: None,
                ty: enum_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumMember,
                task_bound: false,
            },
        );
    }

    enum_ty
}

/// Register a record type without field defaults.
pub(super) fn register_record_type(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
) -> Ty {
    let record_ty = Ty::Record(Arc::new(RecordTy {
        type_params: Vec::new(),
        type_args: Vec::new(),
        name: qualified_name.into(),
        owner_unit: None,
        private_members: Vec::new(),
        fields,
        methods: Vec::new(),
        static_functions: Vec::new(),
        static_procedures: Vec::new(),
    }));
    checker.scopes.imports.define_metadata(
        qualified_name,
        Symbol {
            constant: None,
            ty: record_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );
    record_ty
}

/// Register a host-owned handle using persisted nominal record privacy.
///
/// The private field is declaration metadata only. Runtime handles use
/// `Value::OpaqueHandle`, so FPAS cannot construct or inspect their storage.
pub(super) fn register_opaque_type(checker: &mut Checker, qualified_name: &str) -> Ty {
    let owner = qualified_name
        .rsplit_once('.')
        .map(|(unit, _)| unit.to_string());
    let ty = Ty::Record(Arc::new(RecordTy {
        name: qualified_name.into(),
        owner_unit: owner,
        type_params: Vec::new(),
        type_args: Vec::new(),
        private_members: vec!["_IntrinsicHandle".into()],
        fields: vec![("_IntrinsicHandle".into(), Ty::Integer)],
        methods: Vec::new(),
        static_functions: Vec::new(),
        static_procedures: Vec::new(),
    }));
    checker.scopes.imports.define_metadata(
        qualified_name,
        Symbol {
            constant: None,
            ty: ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );
    ty
}

#[cfg(test)]
#[path = "type_registration_tests.rs"]
mod tests;
