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
            name: (*variant).to_string(),
            fields: vec![],
            backing_value: i64::try_from(index).ok(),
        })
        .collect();
    let enum_ty = Ty::Enum(Arc::new(EnumTy {
        name: qualified_name.into(),
        type_params: Vec::new(),
        type_args: Vec::new(),
        variants: variants.clone(),
    }));
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );

    for variant in &variants {
        let qualified_member = format!("{qualified_name}.{}", variant.name);
        checker.scopes.define(
            &qualified_member,
            Symbol {
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
    register_nominal_type(checker, qualified_name, fields, false)
}

/// Register a host resource whose identity is created by its owning library.
pub(super) fn register_resource_type(checker: &mut Checker, qualified_name: &str) -> Ty {
    register_nominal_type(checker, qualified_name, Vec::new(), true)
}

fn register_nominal_type(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
    is_resource: bool,
) -> Ty {
    let record_ty = Ty::Record(Arc::new(RecordTy {
        name: qualified_name.into(),
        type_params: Vec::new(),
        type_args: Vec::new(),
        is_resource,
        owner_unit: None,
        private_members: Vec::new(),
        fields,
    }));
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: record_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
            task_bound: false,
        },
    );
    record_ty
}
