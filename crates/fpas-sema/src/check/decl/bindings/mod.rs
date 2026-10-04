//! Initialized bindings, local inference and static const classification.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md` and
//! `docs/pascal/language/basics/constants.md`.

mod hoisting;
mod initializers;
mod static_values;

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::BindingDef;

impl Checker {
    /// Check a binding without deriving its type from later statements.
    pub(crate) fn check_binding(&mut self, definition: &BindingDef, mutable: bool) {
        let ty = self.check_binding_initializer(definition);
        self.binding_types.insert(
            (definition.span.source_id, definition.span.offset),
            ty.clone(),
        );
        let is_static =
            !mutable && !ty.is_error() && self.const_expr_is_compile_time_known(&definition.value);
        let task_bound = self.expr_is_task_bound(Self::expr_lookup_key(&definition.value));
        if is_static {
            self.validate_static_operations(&definition.value);
            let value = self
                .try_evaluate_static_value(&definition.value)
                .ok()
                .flatten();
            self.static_constants.insert_binding(definition.span, value);
        }
        if !self.scopes.define_with_declaration(
            &definition.name,
            Symbol {
                ty,
                mutable,
                kind: if is_static {
                    SymbolKind::Const
                } else {
                    SymbolKind::Var
                },
                task_bound,
            },
            definition.span,
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate binding `{}`", definition.name),
                "Each binding name must be unique in the same scope.",
                definition.span,
            );
        }
    }
}
