//! Property assignment checking (`Button.Text := …`).
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_IMMUTABLE_ASSIGNMENT, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

use super::super::context::PropertyWriteInfo;

impl Checker {
    /// Type-check an assignment, including writable property targets.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(crate) fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        // Event/property probes type-check the peeled receiver. On a miss, discard
        // diagnostics from the probe so an undefined base is not reported three times
        // (event attempt + property attempt + normal path).
        let event_checkpoint = self.errors.len();
        if self.try_check_event_assignment(target, value, span) {
            return;
        }
        self.errors.truncate(event_checkpoint);

        let property_checkpoint = self.errors.len();
        if self.try_check_property_assignment(target, value, span) {
            return;
        }
        self.errors.truncate(property_checkpoint);

        let target_ty = self.check_designator_expr(target);
        let value_ty = self.check_expr(value);

        if !target_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "assignment", span);
        }

        let value_is_task_bound = self.expr_is_task_bound(Self::expr_lookup_key(value));
        if target.parts.len() == 1
            && let Some(DesignatorPart::Ident(base, _)) = target.parts.first()
            && let Some(symbol) = self.scopes.lookup_mut(base)
        {
            symbol.task_bound = value_is_task_bound;
        }

        let base_resolved = match target.parts.first() {
            Some(DesignatorPart::Ident(base, _)) => self.scopes.lookup(base).is_some(),
            _ => false,
        };

        if base_resolved && !self.designator_is_mutable_target(target) {
            let target_name = Self::resolve_designator_name(target);
            let hint = match target.parts.first() {
                Some(DesignatorPart::Ident(base, _)) => self
                    .scopes
                    .lookup(base)
                    .map(|symbol| match symbol.kind {
                        SymbolKind::Const => "Constants cannot be reassigned.",
                        SymbolKind::ForVar => "Loop variables are immutable inside the loop body.",
                        SymbolKind::Param => "Mark the parameter `mutable` to allow reassignment.",
                        _ => "Declare with `mutable var` to allow reassignment.",
                    })
                    .unwrap_or("Declare with `mutable var` to allow reassignment."),
                _ => "Declare with `mutable var` to allow reassignment.",
            };

            self.error_with_code(
                SEMA_IMMUTABLE_ASSIGNMENT,
                format!("Cannot assign to `{target_name}`"),
                hint,
                span,
            );
        }
    }

    /// Returns `true` when the assignment target is a record property (handled here).
    fn try_check_property_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
    ) -> bool {
        let Some((property_name, prop_span)) = target.parts.last().and_then(|part| match part {
            DesignatorPart::Ident(name, part_span) => Some((name.as_str(), *part_span)),
            _ => None,
        }) else {
            return false;
        };
        if target.parts.len() < 2 {
            return false;
        }

        // Qualified private locals (`Unit.__private__.Name`) are whole-variable writes.
        // Do not peel the last segment — the prefix is not a standalone designator.
        let only_idents = target
            .parts
            .iter()
            .all(|part| matches!(part, DesignatorPart::Ident(_, _)));
        if only_idents {
            let full_name = Self::resolve_designator_name(target);
            if self.scopes.lookup(&full_name).is_some() {
                return false;
            }
        }

        let receiver = Designator {
            parts: target.parts[..target.parts.len() - 1].to_vec(),
            span: target.span,
        };
        let receiver_key = crate::designator_lookup_key(&receiver);
        let receiver_ty = self.check_designator_expr(&receiver);
        let receiver_reads = self
            .property_reads
            .remove(&receiver_key)
            .unwrap_or_default();
        let resolved = self.resolve_visible_type(&receiver_ty);
        let Ty::Record(record_ty) = resolved else {
            return false;
        };

        if self.reject_private_record_member(&record_ty, property_name, prop_span) {
            let _ = self.check_expr(value);
            return true;
        }

        let Some(property) = self.find_record_property_on_type(&record_ty, property_name) else {
            return false;
        };

        let record_name = record_ty.name.clone();
        let Some(setter) = property.setter.clone() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Property `{record_name}.{property_name}` is read-only"),
                format!(
                    "Read `{property_name}` instead, or add a `write` accessor to the property."
                ),
                prop_span,
            );
            let _ = self.check_expr(value);
            return true;
        };

        let value_ty = self.check_expr(value);
        self.check_type_compat(&property.ty, &value_ty, "property assignment", span);

        let key = crate::designator_lookup_key(target);
        self.property_writes.insert(
            key,
            PropertyWriteInfo {
                setter_name: setter,
                receiver_part_count: target.parts.len() - 1,
                receiver_reads,
            },
        );
        true
    }

    fn find_record_property_on_type(
        &self,
        record_ty: &crate::types::RecordTy,
        member: &str,
    ) -> Option<crate::types::PropertyTy> {
        if let Some((_, property)) = record_ty
            .properties
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
        {
            return Some(property.clone());
        }
        if record_ty.name == "<anonymous>" {
            return None;
        }
        let symbol = self.scopes.lookup(&record_ty.name)?;
        let Ty::Record(canonical) = &symbol.ty else {
            return None;
        };
        canonical
            .properties
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(member))
            .map(|(_, property)| property.clone())
    }
}
