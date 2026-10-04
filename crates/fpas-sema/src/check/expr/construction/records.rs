//! Named record construction, sharing visibility and required-field validation.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use crate::check::Checker;
use crate::scope::SymbolKind;
use crate::types::{ParamTy, Ty};
use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_parser::{DesignatorPart, Expr};

impl Checker {
    /// Check explicit field syntax or an empty call whose target is a record type.
    pub(in crate::check) fn try_check_record_construction(
        &mut self,
        expression: &Expr,
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        let (source_name, fields, positional) = match expression {
            Expr::RecordConstruction {
                type_name, fields, ..
            } => (type_name.parts.join("."), fields.as_slice(), &[][..]),
            Expr::Call {
                designator, args, ..
            } if designator
                .parts
                .iter()
                .all(|part| matches!(part, DesignatorPart::Ident(..))) =>
            {
                let source_name = Self::designator_name(designator);
                let name = self.qualified_import_name(&source_name);
                if !self.scopes.lookup(&name).is_some_and(|symbol| {
                    symbol.kind == SymbolKind::Type && matches!(symbol.ty, Ty::Record(_))
                }) {
                    return None;
                }
                (source_name, &[][..], args.as_slice())
            }
            _ => return None,
        };
        let span = expression.span();
        let name = self.resolve_source_name(&source_name, span);
        let Some(symbol) = self.scopes.lookup(&name).cloned() else {
            self.error_with_code(SEMA_UNKNOWN_NAME, format!("Unknown record type `{source_name}`"), "Use the name of a declared record type, qualified by its import alias when imported.", span);
            for field in fields {
                self.check_expr(&field.value);
            }
            return Some(Ty::Error);
        };
        if symbol.kind != SymbolKind::Type || !matches!(symbol.ty, Ty::Record(_)) {
            self.error_with_code(SEMA_TYPE_MISMATCH, format!("Named fields require a record type; `{source_name}` is not a record construction target"),
                "Routine and variant arguments are positional. For a record, write `TypeName(Field := Value)`.", span);
            for field in fields {
                self.check_expr(&field.value);
            }
            return Some(Ty::Error);
        }
        let Ty::Record(template) = &symbol.ty else {
            unreachable!()
        };
        if template.is_resource {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Resource type `{source_name}` cannot be structurally constructed"),
                "Use a public factory from the declaring unit.",
                span,
            );
            for field in fields {
                self.check_expr(&field.value);
            }
            return Some(Ty::Error);
        }
        if self.reject_private_record_construction(template, span) {
            for field in fields {
                self.check_expr(&field.value);
            }
            return Some(Ty::Error);
        }
        if !positional.is_empty() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Record construction `{source_name}` requires named fields"),
                "Write `TypeName(Field := Value, ...)` instead of positional arguments.",
                span,
            );
            self.check_args_only(positional);
            return Some(Ty::Error);
        }
        self.validate_unique_record_fields(fields, "record construction");
        let mut inferred = self.construction_bindings(&symbol.ty, expected);
        let contextual = inferred.clone();
        let mut parameters = Vec::new();
        let mut actual_types = Vec::new();
        let mut unknown_field = false;
        for field in fields {
            if let Some((_, declared)) = template
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
            {
                let field_expected =
                    self.construction_payload_context(&symbol.ty, declared, &inferred);
                let actual = self.check_expr_with_expected(&field.value, &field_expected);
                parameters.push(ParamTy {
                    mutable: false,
                    name: field.name.clone(),
                    ty: declared.substitute(&contextual),
                });
                actual_types.push(actual);
            } else {
                self.error_with_code(
                    SEMA_UNKNOWN_NAME,
                    format!("Record type `{source_name}` has no field `{}`", field.name),
                    format!(
                        "Known fields: {}. Remove the unknown field or fix the name.",
                        template
                            .fields
                            .iter()
                            .map(|(name, _)| name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    field.span,
                );
                self.check_expr(&field.value);
                unknown_field = true;
            }
        }
        if unknown_field {
            return Some(Ty::Error);
        }
        for (parameter, actual) in parameters.iter().zip(&actual_types) {
            self.collect_type_param_bindings(&parameter.ty, actual, &mut inferred, span);
        }
        let constructed = self.finish_data_construction(&symbol.ty, &inferred, &source_name, span);
        if let Ty::Record(record) = &constructed {
            self.validate_record_construction_fields(fields, record, span);
        }
        let key = Self::expr_lookup_key(expression);
        self.record_constructions.insert(key);
        self.expr_types.insert(key, constructed.clone());
        self.propagate_task_bound_expr(expression, key);
        Some(constructed)
    }
}
