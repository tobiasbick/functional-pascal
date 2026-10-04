use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_UNKNOWN_TYPE};
use fpas_parser::{TypeBody, TypeDef};
use std::sync::Arc;

pub(in crate::check) mod collection;
mod enums;
mod generics;
mod inhabitation;
mod records;

impl Checker {
    pub(super) fn check_type_def(&mut self, td: &TypeDef) {
        self.with_type_params(&td.type_params, td.span, |checker| {
            checker.check_type_body(td);
        });
    }

    fn check_type_body(&mut self, td: &TypeDef) {
        if !self.type_collection.collecting && self.has_collected_type(td) {
            if let TypeBody::Record(record) = &td.body {
                self.check_record_type_def(td, record);
            }
            return;
        }
        match &td.body {
            TypeBody::Record(record) => self.check_record_type_def(td, record),
            TypeBody::Enum(enum_ty) => self.check_enum_type_def(td, enum_ty),
            TypeBody::Alias(type_expr) => self.check_alias_type_def(td, type_expr),
        }
    }

    fn check_alias_type_def(&mut self, td: &TypeDef, type_expr: &fpas_parser::TypeExpr) {
        if !td.type_params.is_empty() {
            self.error_with_code(SEMA_UNKNOWN_TYPE,
                "Generic type parameters require a record or enum declaration",
                "Use `type Box of (T) = record Value: T; end record;`, or alias a concrete application such as `type IntegerBox = Box of (integer);`.", td.span);
            return;
        }
        let ty = self.resolve_type_expr(type_expr);
        if !self.define_type_symbol(td, ty.clone()) {
            return;
        }

        if let Ty::Enum(enum_ty) = ty {
            self.register_enum_alias_variant_symbols(td, &enum_ty);
        }
    }

    /// Expose qualified enum variants through an alias without adding ambiguous short names.
    fn register_enum_alias_variant_symbols(&mut self, td: &TypeDef, enum_ty: &Arc<EnumTy>) {
        for variant in &enum_ty.variants {
            let kind = if variant.fields.is_empty() {
                SymbolKind::EnumMember
            } else {
                SymbolKind::EnumVariantConstructor
            };
            let qualified = format!("{}.{}", td.name, variant.name);
            if !self.scopes.define_in_root(
                &qualified,
                Symbol {
                    ty: Ty::Enum(enum_ty.clone()),
                    mutable: false,
                    kind,
                    task_bound: false,
                },
            ) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate enum member `{qualified}`"),
                    "Each enum member name must be unique in the program.",
                    td.span,
                );
            }
        }
    }

    pub(super) fn define_type_symbol(&mut self, td: &TypeDef, ty: Ty) -> bool {
        if self.has_collected_type(td) {
            if let Some(symbol) = self.scopes.lookup_root_mut(&td.name) {
                *symbol.ty_mut() = ty;
            }
            return true;
        }
        if self.scopes.define(
            &td.name,
            Symbol {
                ty,
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
            },
        ) {
            return true;
        }

        self.error_with_code(
            SEMA_DUPLICATE_DECLARATION,
            format!("Duplicate type `{}`", td.name),
            "Each name must be unique in the same scope.",
            td.span,
        );
        false
    }
}
