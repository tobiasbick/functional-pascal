//! Record type checking.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`,
//! `docs/pascal/language/types/record-methods.md`

mod methods;
mod signatures;

use super::Checker;
use crate::scope::{Symbol, SymbolKind, canonical_symbol_name};
use crate::types::{RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{RecordType, TypeDef, Visibility};
use std::collections::HashSet;
use std::sync::Arc;

impl Checker {
    pub(super) fn check_record_type_def(&mut self, td: &TypeDef, record: &RecordType) {
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
                task_bound: false,
            },
        ) {
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
                    "Each field, method, static routine, property, and event name must be unique within the record type.",
                    field.span,
                );
                continue;
            }
            field_indexes.push(field_index);
            fields.push((field.name.clone(), self.resolve_type_expr(&field.type_expr)));
        }

        // Validate default values and build the defaults map entry.
        let defaults_entry: Vec<(String, Option<fpas_parser::Expr>)> = field_indexes
            .iter()
            .map(|field_index| &record.fields[*field_index])
            .zip(fields.iter())
            .map(|(field_def, (_, field_ty))| {
                if let Some(default_expr) = &field_def.default_value {
                    let default_ty = self.check_expr(default_expr);
                    self.check_type_compat(
                        field_ty,
                        &default_ty,
                        &format!("default value for field `{}`", field_def.name),
                        field_def.span,
                    );
                    (field_def.name.clone(), Some(default_expr.clone()))
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
                .chain(
                    record
                        .methods
                        .iter()
                        .filter(|method| method.visibility() == Visibility::Private)
                        .map(|method| method.name().to_string()),
                )
                .chain(
                    record
                        .properties
                        .iter()
                        .filter(|property| property.visibility == Visibility::Private)
                        .map(|property| property.name.clone()),
                )
                .chain(
                    record
                        .events
                        .iter()
                        .filter(|event| event.visibility == Visibility::Private)
                        .map(|event| event.name.clone()),
                )
                .collect()
        } else {
            Vec::new()
        };
        let record_ty = RecordTy {
            name: td.name.clone(),
            owner_unit,
            private_members,
            fields,
            methods: Vec::new(),
            static_functions: Vec::new(),
            static_procedures: Vec::new(),
            properties: Vec::new(),
            events: Vec::new(),
        };
        let mut ty = Ty::Record(Arc::new(record_ty));

        let (members, pending_bodies) =
            self.check_record_methods(&td.name, &ty, &record.methods, &mut seen_members);

        if let Ty::Record(record_ty) = &mut ty {
            let record_ty = Arc::make_mut(record_ty);
            record_ty.methods = members.instance_methods;
            record_ty.static_functions = members.static_functions;
            record_ty.static_procedures = members.static_procedures;
        }

        let properties =
            self.check_record_properties(&td.name, &ty, &record.properties, &mut seen_members);
        if let Ty::Record(record_ty) = &mut ty {
            let record_ty = Arc::make_mut(record_ty);
            record_ty.properties = properties;
        }

        let events = self.check_record_events(&td.name, &ty, &record.events, &mut seen_members);
        if let Ty::Record(record_ty) = &mut ty {
            let record_ty = Arc::make_mut(record_ty);
            record_ty.events = events;
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing.ty_mut() = ty;
        }

        // Method bodies run after properties and events are visible on the type symbol.
        for pending in pending_bodies {
            self.check_method_body(
                &pending.qualified_name,
                pending.type_params,
                &pending.params,
                &pending.param_spans,
                pending.return_type,
                pending.body,
            );
        }
    }
}
