use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy, Ty};
use fpas_lexer::Span;
use fpas_parser::Expr;

/// Register a simple enum type and expose each variant as a qualified enum member.
///
/// **Documentation:** `docs/pascal/05-types.md`, `docs/pascal/std/tui.md` (from the repository root).
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
    let member_names: Vec<String> = variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect();
    let enum_ty = Ty::Enum(EnumTy {
        name: qualified_name.into(),
        variants,
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: enum_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );

    for member_name in &member_names {
        let qualified_member = format!("{qualified_name}.{member_name}");
        checker.scopes.define(
            &qualified_member,
            Symbol {
                ty: enum_ty.clone(),
                mutable: false,
                kind: SymbolKind::EnumMember,
            },
        );
    }

    enum_ty
}

/// Register a record type without field defaults.
///
/// **Documentation:** `docs/pascal/05-types.md`, `docs/pascal/std/tui.md` (from the repository root).
pub(super) fn register_record_type(
    checker: &mut Checker,
    qualified_name: &str,
    fields: Vec<(String, Ty)>,
) -> Ty {
    let record_ty = Ty::Record(RecordTy {
        name: qualified_name.into(),
        fields,
        methods: Vec::new(),
    });
    checker.scopes.define(
        qualified_name,
        Symbol {
            ty: record_ty.clone(),
            mutable: false,
            kind: SymbolKind::Type,
        },
    );
    record_ty
}

/// Register a record type and store semantic defaults for fields that have built-in values.
///
/// **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).
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

/// Build the semantic default for optional TUI handlers.
///
/// **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).
pub(super) fn default_none_expr() -> Expr {
    Expr::OptionNone(builtin_span())
}

/// Build the semantic default for integer TUI handler fields.
///
/// **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).
pub(super) fn default_zero_expr() -> Expr {
    Expr::Integer(0, builtin_span())
}

fn builtin_span() -> Span {
    Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    }
}
