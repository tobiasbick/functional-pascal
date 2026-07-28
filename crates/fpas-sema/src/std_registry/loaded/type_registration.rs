//! Shared std unit type registration helpers.
//!
//! **Documentation:** `docs/pascal/language/types/README.md`

use crate::check::Checker;
use crate::check::spans::synthetic_span;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use fpas_parser::Expr;
use std::sync::Arc;

/// Register a simple enum type and expose each variant as a qualified enum member.
pub(super) fn register_enum_type(
    checker: &mut Checker,
    qualified_name: &str,
    variants: &[&str],
) -> Ty {
    let variants: Vec<EnumVariantTy> = variants
        .iter()
        .map(|variant| EnumVariantTy {
            name: (*variant).to_string(),
            fields: vec![],
        })
        .collect();
    let enum_ty = Ty::Enum(Arc::new(EnumTy {
        name: qualified_name.into(),
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
    let record_ty = Ty::Record(Arc::new(RecordTy {
        name: qualified_name.into(),
        owner_unit: None,
        private_members: Vec::new(),
        fields,
        methods: Vec::new(),
        static_functions: Vec::new(),
        static_procedures: Vec::new(),
        properties: Vec::new(),
        events: Vec::new(),
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

/// Register a record type and store semantic defaults for fields that have built-in values.
pub(super) fn register_record_type_with_defaults(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
    defaults: Vec<(String, Option<Expr>)>,
) -> Ty {
    let record_ty = register_record_type(checker, qualified_name, fields);
    if defaults.iter().any(|(_, default)| default.is_some()) {
        checker
            .record_defaults
            .insert(qualified_name.to_string(), defaults);
    }
    record_ty
}

/// Build the semantic default for optional handler fields.
pub(super) fn default_none_expr() -> Expr {
    Expr::OptionNone(synthetic_span())
}

/// Build the semantic default for integer handler fields.
pub(super) fn default_zero_expr() -> Expr {
    Expr::Integer(0, synthetic_span())
}

/// Look up a type symbol registered earlier in std unit loading.
pub(super) fn lookup_required_type(checker: &Checker, qualified_name: &str, message: &str) -> Ty {
    checker
        .scopes
        .lookup(qualified_name)
        .map(|symbol| symbol.ty.clone())
        .unwrap_or_else(|| unreachable!("{message}"))
}
