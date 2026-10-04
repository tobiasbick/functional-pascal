//! Record type checking.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use super::Checker;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::{RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{RecordType, TypeDef, Visibility};
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    pub(super) fn check_record_type_def(&mut self, td: &TypeDef, record: &RecordType) {
        if !self.has_collected_type(td)
            && !self.scopes.define(
                &td.name,
                Symbol {
                    ty: Ty::Named(td.name.clone()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            )
        {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate type `{}`", td.name),
                "Each name must be unique in the same scope.",
                td.span,
            );
            return;
        }

        let mut seen_members = HashSet::new();
        let mut field_indexes = Vec::new();
        let mut fields = Vec::new();
        for (field_index, field) in record.fields.iter().enumerate() {
            if !seen_members.insert(canonical_symbol_name(&field.name)) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate record member `{}`", field.name),
                    "Each field name must be unique within the record type.",
                    field.span,
                );
                continue;
            }
            field_indexes.push(field_index);
            fields.push((field.name.clone(), self.resolve_type_expr(&field.type_expr)));
        }

        // Validate default values and build the defaults map entry.
        let defaults_entry: Vec<(String, Option<crate::RecordDefault>)> = field_indexes
            .iter()
            .map(|field_index| &record.fields[*field_index])
            .zip(fields.iter())
            .map(|(field_def, (_, field_ty))| {
                if let Some(default_expr) = &field_def.default_value {
                    if self.type_collection.collecting {
                        return (
                            field_def.name.clone(),
                            Some(crate::RecordDefault::Expression(default_expr.clone())),
                        );
                    }
                    let previous =
                        self.pure_evaluation
                            .replace(crate::check::purity::PureEvaluation {
                                scope_index: self.scopes.scope_count(),
                            });
                    let previous_default =
                        self.begin_default_purity(&td.name, &field_def.name, &td.type_params);
                    let default_ty = self.check_expr_with_expected(default_expr, field_ty);
                    self.finish_default_purity(previous_default);
                    self.pure_evaluation = previous;
                    self.check_type_compat(
                        field_ty,
                        &default_ty,
                        &format!("default value for field `{}`", field_def.name),
                        field_def.span,
                    );
                    if !default_ty.is_error() && self.const_expr_is_compile_time_known(default_expr)
                    {
                        self.validate_static_operations(default_expr);
                    }
                    (
                        field_def.name.clone(),
                        Some(crate::RecordDefault::Expression(default_expr.clone())),
                    )
                } else {
                    (field_def.name.clone(), None)
                }
            })
            .collect();

        // Only register defaults when at least one field has a default, since the
        // compiler uses the absence of an entry to mean "no defaults, emit as-is".
        if defaults_entry.iter().any(|(_, default)| default.is_some()) {
            self.record_defaults.insert(td.name.clone(), defaults_entry);
        }

        let owner_unit = self
            .scopes
            .function_ctx
            .as_ref()
            .and_then(|context| context.owner_unit.clone());
        let private_members = if owner_unit.is_some() {
            record
                .fields
                .iter()
                .filter(|field| field.visibility == Visibility::Private)
                .map(|field| field.name.clone())
                .collect()
        } else {
            Vec::new()
        };
        let record_ty = RecordTy {
            name: td.name.clone(),
            type_params: self.resolve_type_params(&td.type_params),
            type_args: Vec::new(),
            is_resource: false,
            owner_unit,
            private_members,
            fields,
        };
        let ty = Ty::Record(Arc::new(record_ty));

        if let Some(existing) = self.scopes.lookup_root_mut(&td.name) {
            *existing.ty_mut() = ty;
        }
    }
}
