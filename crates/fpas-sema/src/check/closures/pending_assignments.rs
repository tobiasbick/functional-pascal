//! Mutable writes cannot retain immutable initializer provenance.
//! See `docs/pascal/language/functions/closures.md`.

use super::pending_capabilities::binding_key;
use crate::{TransferCapability, check::Checker};
use fpas_parser::{Designator, DesignatorPart, Expr};
use std::collections::HashSet;

impl Checker {
    /// A still-incomplete named callable does not prove a transferable mutable value.
    pub(crate) fn assignment_value_capability(&self, value: &Expr) -> TransferCapability {
        let capability = self.expr_capability(Self::expr_lookup_key(value));
        let mut pending = self.pending_expr_captures(value);
        let mut visited = HashSet::new();
        while let Some(target) = pending.pop() {
            if !visited.insert(target) {
                continue;
            }
            if !self.nested_routine_captures.contains_key(&target) {
                return capability.join(TransferCapability::Unknown);
            }
            if let Some(targets) = self.pending_routine_captures.get(&target) {
                pending.extend(targets);
            }
        }
        capability
    }

    /// A whole-binding overwrite ends the initializer's deferred provenance.
    ///
    /// New incomplete assignments use Unknown and the runtime transfer guard.
    /// Keeping old targets would reject a later overwrite with a proven free value.
    pub(crate) fn forget_overwritten_capture_provenance(&mut self, target: &Designator) {
        let Some((_, root_parts)) = self.designator_root_symbol(&target.parts) else {
            return;
        };
        if root_parts != target.parts.len() {
            return;
        }
        let name = target
            .parts
            .iter()
            .filter_map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(".");
        let Some((_, _, Some(span))) = self.scopes.lookup_with_scope_and_declaration(&name) else {
            return;
        };
        self.capture_dependencies
            .capabilities
            .bindings
            .remove(&binding_key(&name, span));
    }
}
