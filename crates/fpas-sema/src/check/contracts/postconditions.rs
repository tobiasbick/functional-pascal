//! Clause-local return bindings use the declared result type and heading scope.
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::Ty;
use fpas_parser::EnsuresClause;

impl Checker {
    /// Check each exit clause independently before introducing body-local declarations.
    pub(crate) fn check_postconditions(&mut self, clauses: &[EnsuresClause]) {
        let previous = self.contracts.scope.replace(self.scopes.scope_count() - 1);
        for clause in clauses {
            self.scopes.push_scope();
            if let Some((name, span)) = &clause.result_name {
                self.check_import_alias_collision(name, *span);
                if let Some(ty) = self
                    .scopes
                    .function_ctx
                    .as_ref()
                    .and_then(|context| context.return_type.clone())
                {
                    self.scopes.define_with_declaration(
                        name,
                        Symbol {
                            constant: None,
                            capability: self.capability_for_type(&ty),
                            ty,
                            mutable: false,
                            kind: SymbolKind::Const,
                        },
                        *span,
                    );
                } else {
                    self.reject_contract_expression(
                        "a procedure has no return value to name",
                        *span,
                    );
                }
            }
            let ty = self.check_expr_with_expected(&clause.condition, Some(&Ty::Boolean));
            self.check_type_compat(&Ty::Boolean, &ty, "postcondition", clause.condition.span());
            self.check_contract_expression(&clause.condition, clause.span);
            self.scopes.pop_scope();
        }
        self.contracts.scope = previous;
    }
}
