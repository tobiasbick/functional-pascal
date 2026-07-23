use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::std_units::canonical_unit_from_uses_clause;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_parser::{Program, Unit};
use fpas_unit::interface::UnitInterface;

impl Checker {
    pub fn check_program(&mut self, program: &Program) {
        self.prepare_program(program);

        for decl in &program.declarations {
            self.check_decl(decl);
        }

        self.check_program_body(program);
    }

    pub(crate) fn check_program_with_interfaces(
        &mut self,
        program: &Program,
        interfaces: &[UnitInterface],
        supporting_interfaces: &[UnitInterface],
    ) -> Result<(), crate::InterfaceConversionError> {
        let dependency_names: std::collections::HashSet<String> = interfaces
            .iter()
            .map(|interface| interface.unit_name.to_ascii_lowercase())
            .collect();
        let intrinsic_uses: Vec<_> = program
            .uses
            .iter()
            .filter(|used| !dependency_names.contains(&used.parts.join(".").to_ascii_lowercase()))
            .cloned()
            .collect();
        self.prepare_uses(&intrinsic_uses);
        self.install_supporting_interface_types(supporting_interfaces)?;
        self.install_interfaces(program, interfaces)?;

        for decl in &program.declarations {
            self.check_decl(decl);
        }

        self.check_program_body(program);
        Ok(())
    }

    pub(crate) fn check_unit_with_interfaces(
        &mut self,
        unit: &Unit,
        interfaces: &[UnitInterface],
        supporting_interfaces: &[UnitInterface],
    ) -> Result<(), crate::InterfaceConversionError> {
        let dependency_names: std::collections::HashSet<String> = interfaces
            .iter()
            .map(|interface| interface.unit_name.to_ascii_lowercase())
            .collect();
        let intrinsic_uses: Vec<_> = unit
            .uses
            .iter()
            .filter(|used| !dependency_names.contains(&used.parts.join(".").to_ascii_lowercase()))
            .cloned()
            .collect();
        self.prepare_uses(&intrinsic_uses);
        self.install_supporting_interface_types(supporting_interfaces)?;
        self.install_interfaces_for_declarations(&unit.declarations, interfaces)?;

        let previous_context = self.scopes.function_ctx.take();
        self.scopes.function_ctx = Some(FunctionCtx {
            name: unit.name.parts.join("."),
            return_type: None,
            owner_unit: Some(unit.name.parts.join(".")),
        });
        for declaration in &unit.declarations {
            self.check_decl(declaration);
        }
        self.scopes.function_ctx = previous_context;
        Ok(())
    }

    fn prepare_program(&mut self, program: &Program) {
        self.prepare_uses(&program.uses);
    }

    fn prepare_uses(&mut self, uses: &[fpas_parser::QualifiedId]) {
        self.loaded_std_units.clear();
        self.short_builtin_redirect.clear();
        self.std_short_alias_keys.clear();
        self.ambiguous_enum_variants.clear();
        self.enum_short_variant_keys.clear();
        for u in uses {
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
    }

    fn check_program_body(&mut self, program: &Program) {
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
