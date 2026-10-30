//! Assignment statement checking.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`

use super::Checker;
use crate::TransferCapability;
use crate::scope::SymbolKind;
use fpas_diagnostics::codes::SEMA_IMMUTABLE_ASSIGNMENT;
use fpas_lexer::Span;
use fpas_parser::{Designator, DesignatorPart, Expr};

impl Checker {
    /// Type-check an assignment and its target mutability.
    ///
    /// **Documentation:** `docs/pascal/language/basics/variables.md`
    pub(crate) fn check_assign_stmt(&mut self, target: &Designator, value: &Expr, span: Span) {
        if self.report_function_name_assignment(target, value, span) {
            return;
        }
        let target_ty = self.check_designator_expr(target);
        let value_ty = self.check_expr_with_expected(value, Some(&target_ty));

        if !target_ty.is_error() {
            self.check_type_compat(&target_ty, &value_ty, "assignment", span);
            if self.reject_read_only_index_assignment(target) {
                return;
            }
        }

        let capability = if self.type_can_contain_callable(&target_ty) {
            self.assignment_value_capability(value)
        } else {
            TransferCapability::Transferable
        };
        self.update_designator_capability(target, capability, true);
        self.forget_overwritten_capture_provenance(target);

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

    /// String and byte-buffer indexed reads do not denote writable scalar storage.
    fn reject_read_only_index_assignment(&mut self, target: &Designator) -> bool {
        let Some((span, operation)) = self.designator_read_only_index(target) else {
            return false;
        };
        self.error_with_code(
            SEMA_IMMUTABLE_ASSIGNMENT,
            "Indexed scalar slots are read-only",
            format!("Use the writable receiver's {operation} operation."),
            span,
        );
        true
    }

    /// Conservatively retain callable facts when a native or `var` call mutates storage.
    pub(crate) fn join_designator_root_capability(
        &mut self,
        target: &Designator,
        capability: TransferCapability,
    ) {
        self.update_designator_capability(target, capability, false);
    }

    fn update_designator_capability(
        &mut self,
        target: &Designator,
        capability: TransferCapability,
        overwrite_whole_binding: bool,
    ) {
        let Some((_, root_parts)) = self.designator_root_symbol(&target.parts) else {
            return;
        };
        let root = target.parts[..root_parts]
            .iter()
            .filter_map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(".");
        if let Some(symbol) = self.scopes.lookup_mut(&root) {
            symbol.capability = if overwrite_whole_binding && root_parts == target.parts.len() {
                capability
            } else {
                // Replacing one path does not clear local values at another path.
                symbol.capability.join(capability)
            };
        }
    }
}
