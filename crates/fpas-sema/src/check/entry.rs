use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::std_units::canonical_unit_from_uses_clause;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_parser::Program;

impl Checker {
    pub fn check_program(&mut self, program: &Program) {
        self.loaded_std_units.clear();
        self.short_builtin_redirect.clear();
        self.std_short_alias_keys.clear();
        self.ambiguous_enum_variants.clear();
        self.enum_short_variant_keys.clear();
        for u in &program.uses {
            match canonical_unit_from_uses_clause(u) {
                Ok(canon) => {
                    self.loaded_std_units.insert(canon);
                }
                Err(msg) => {
                    self.error_with_code(
                        SEMA_UNKNOWN_NAME,
                        msg,
                        format!(
                            "Use one of: {}.",
                            crate::std_units::std_units_list_for_hint()
                        ),
                        u.span,
                    );
                }
            }
        }

        self.register_primitive_types();
        self.register_loaded_std_library();

        for decl in &program.declarations {
            self.check_decl(decl);
        }

        let prev_ctx = self.scopes.function_ctx.take();
        self.scopes.function_ctx = Some(FunctionCtx {
            name: program.name.clone(),
            return_type: None,
            owner_unit: None,
        });

        // Program-body locals live in a non-root scope so closures can capture them.
        // Module-level declarations remain in scope 0 and are not stored in closure environments.
        self.scopes.push_scope();
        for stmt in &program.body {
            self.check_stmt(stmt);
        }
        self.scopes.pop_scope();

        self.scopes.function_ctx = prev_ctx;
    }

    fn register_primitive_types(&mut self) {
        for name in &["integer", "real", "boolean", "string"] {
            self.scopes.define(
                name,
                Symbol {
                    ty: Ty::Named(name.to_string()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }
    }

    /// Symbols from standard units that are actually in scope (requires matching `uses`).
    fn register_loaded_std_library(&mut self) {
        crate::std_registry::register_loaded_std(self);
        crate::std_registry::register_short_aliases(self);
    }
}
