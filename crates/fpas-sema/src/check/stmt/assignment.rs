//! Assignment statement checking, including record event targets.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::{RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};
use std::sync::Arc;

/// Record member named by the last segment of an assignment target.
pub(super) struct MemberAssignmentTarget<'a> {
    /// Visible record type of the receiver.
    pub(super) record_ty: Arc<RecordTy>,
    /// Member name as written in the target.
    pub(super) name: &'a str,
    /// Source span of the member segment.
    pub(super) span: Span,
}

impl Checker {
    /// Type-check an assignment, including event targets.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(crate) fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        // The member probe type-checks the peeled receiver. On a miss, discard its
        // diagnostics so an undefined base is reported only once by the normal path.
        let checkpoint = self.errors.len();
        if self.try_check_member_assignment(target, value, span) {
            return;
        }
        self.errors.truncate(checkpoint);

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

        if let Some((symbol, _)) = self.designator_root_symbol(&target.parts)
            && !self.designator_is_mutable_target(target)
        {
            let target_name = Self::resolve_designator_name(target);
            let hint = match symbol.kind {
                SymbolKind::Const => {
                    "A `const` binding cannot be changed; declare it with `var` to allow reassignment."
                }
                SymbolKind::ForVar => "Loop variables are immutable inside the loop body.",
                SymbolKind::Param => {
                    "Parameters are read-only; start the body with a local copy, for example `var LocalValue: integer := Value;`."
                }
                _ => "Declare with `var` to allow reassignment.",
            };

            self.error_with_code(
                SEMA_IMMUTABLE_ASSIGNMENT,
                format!("Cannot assign to `{target_name}`"),
                hint,
                span,
            );
        }
    }

    /// Returns `true` when the target is a record event (handled here).
    fn try_check_member_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        span: Span,
    ) -> bool {
        let Some(member) = self.resolve_member_assignment_target(target) else {
            return false;
        };

        if self.reject_private_record_member(&member.record_ty, member.name, member.span) {
            let _ = self.check_expr(value);
            return true;
        }

        if let Some(event) = self.find_record_event_on_type(&member.record_ty, member.name) {
            self.check_event_assignment(target, value, span, event);
            return true;
        }

        false
    }

    /// Peels the last target segment and resolves the receiver to a record type.
    fn resolve_member_assignment_target<'a>(
        &mut self,
        target: &'a Designator,
    ) -> Option<MemberAssignmentTarget<'a>> {
        let (name, span) = match target.parts.last()? {
            DesignatorPart::Ident(name, part_span) => (name.as_str(), *part_span),
            _ => return None,
        };
        if target.parts.len() < 2 {
            return None;
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
                return None;
            }
        }

        let receiver = Designator {
            parts: target.parts[..target.parts.len() - 1].to_vec(),
            span: target.span,
        };
        let receiver_ty = self.check_designator_expr(&receiver);
        let Ty::Record(record_ty) = self.resolve_visible_type(&receiver_ty) else {
            return None;
        };

        Some(MemberAssignmentTarget {
            record_ty,
            name,
            span,
        })
    }

    /// Finds a record member by case-insensitive name, falling back to the canonical
    /// declaration of a named record type.
    pub(super) fn find_record_member_on_type<T: Clone>(
        &self,
        record_ty: &RecordTy,
        member: &str,
        members: fn(&RecordTy) -> &[(String, T)],
    ) -> Option<T> {
        let find = |record: &RecordTy| {
            members(record)
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(member))
                .map(|(_, value)| value.clone())
        };
        if let Some(found) = find(record_ty) {
            return Some(found);
        }
        let symbol = self.scopes.lookup(&record_ty.name)?;
        let Ty::Record(canonical) = &symbol.ty else {
            return None;
        };
        find(canonical)
    }
}
