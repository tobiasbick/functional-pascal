//! Conservative callable-storage invalidation without an interprocedural effects model.
//! See `docs/pascal/language/functions/closures.md`.

use crate::{TransferCapability, check::Checker};
use fpas_parser::Expr;

impl Checker {
    /// An outer mutable value may differ each time this routine is invoked.
    pub(crate) fn forget_outer_mutable_capabilities(&mut self) {
        for (name, declaration, ty) in self.scopes.mutable_bindings() {
            if self.type_can_contain_callable(&ty) {
                self.scopes.set_capture_capability(
                    &name,
                    declaration,
                    TransferCapability::Unknown,
                    false,
                );
            }
        }
    }

    /// Local or unknown callees and callbacks may write captured callable storage.
    ///
    /// The existing capture graph identifies possible storage roots. Without a
    /// write-effect summary, discard their transferable proofs conservatively;
    /// known local evidence remains valid across unknown writes.
    pub(crate) fn invalidate_captured_callable_writes(&mut self, name: &str, args: &[&Expr]) {
        let source_call = self
            .scopes
            .lookup(name)
            .is_none_or(|symbol| symbol.kind != crate::scope::SymbolKind::BuiltinStd);
        let has_callable_arguments = args.iter().any(|arg| {
            self.expr_types
                .get(&Self::expr_lookup_key(arg.argument_value()))
                .is_some_and(|ty| self.type_can_contain_callable(ty))
        });
        if source_call || has_callable_arguments {
            for (name, declaration, ty) in self.scopes.mutable_global_bindings() {
                if self.type_can_contain_callable(&ty) {
                    self.scopes.set_capture_capability(
                        &name,
                        declaration,
                        TransferCapability::Unknown,
                        true,
                    );
                }
            }
        }
        let callable = self
            .scopes
            .lookup(name)
            .map_or(TransferCapability::Unknown, |symbol| symbol.capability);
        // A transferable routine value may still invoke a stored local callback.
        // Imported routines have the same obligation as local source routines.
        if !source_call
            && !has_callable_arguments
            && callable == TransferCapability::Transferable
            && args.iter().all(|arg| {
                self.expr_capability(Self::expr_lookup_key(arg.argument_value()))
                    == TransferCapability::Transferable
            })
        {
            return;
        }
        let roots: Vec<_> = self
            .closure_infos
            .values()
            .flat_map(|info| &info.captures)
            .chain(
                self.nested_routine_captures
                    .values()
                    .flat_map(|info| &info.captures),
            )
            .filter(|capture| capture.mutable && self.type_can_contain_callable(&capture.ty))
            .map(|capture| (capture.name.clone(), capture.declaration))
            .collect();
        for (name, declaration) in roots {
            self.scopes.set_capture_capability(
                &name,
                declaration,
                TransferCapability::Unknown,
                true,
            );
        }
    }
}
