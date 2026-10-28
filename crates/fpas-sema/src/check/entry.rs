use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::std_units::canonical_unit_from_uses_clause;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_UNKNOWN_NAME;
use fpas_parser::{Program, Unit};
use fpas_unit::interface::UnitInterface;

impl Checker {
    /// Collect lexical signatures, check ordered initialization, and validate the program body.
    pub fn check_program(&mut self, program: &Program) {
        self.prepare_program(program);
        self.collect_unit_types(&program.declarations);

        self.check_scope_declarations(&program.declarations);

        self.check_program_body(program);
        self.finish_pending_capture_checks();
        self.check_task_handle_escapes(&program.declarations, Some(&program.body));
        self.validate_dictionary_keys();
        self.finish_contract_checks();
        self.finish_deferred_type_arguments();
        self.check_initialization(&program.declarations, Some(&program.body));
    }

    /// Install dependency interfaces and check scope-wide signatures with ordered values.
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
            .filter(|used| {
                fpas_builtins::is_retired_type_helper_unit(&used.unit.parts.join("."))
                    || !dependency_names.contains(&used.unit.parts.join(".").to_ascii_lowercase())
            })
            .cloned()
            .collect();
        self.prepare_uses(&program.uses, &intrinsic_uses, None);
        self.used_unit_names = program
            .uses
            .iter()
            .map(|used| used.unit.parts.join(".").to_ascii_lowercase())
            .collect();
        self.install_supporting_interface_types(supporting_interfaces)?;
        self.install_interfaces(interfaces)?;
        self.collect_unit_types(&program.declarations);

        self.check_scope_declarations(&program.declarations);

        self.check_program_body(program);
        self.finish_pending_capture_checks();
        self.check_task_handle_escapes(&program.declarations, Some(&program.body));
        self.validate_dictionary_keys();
        self.finish_contract_checks();
        self.finish_deferred_type_arguments();
        self.check_initialization(&program.declarations, Some(&program.body));
        Ok(())
    }

    /// Check a unit with scope-wide signatures and proved source-order initialization.
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
            .filter(|used| {
                fpas_builtins::is_retired_type_helper_unit(&used.unit.parts.join("."))
                    || !dependency_names.contains(&used.unit.parts.join(".").to_ascii_lowercase())
            })
            .cloned()
            .collect();
        self.prepare_uses(
            &unit.uses,
            &intrinsic_uses,
            unit.name.parts.first().map(String::as_str),
        );
        self.used_unit_names = unit
            .uses
            .iter()
            .map(|used| used.unit.parts.join(".").to_ascii_lowercase())
            .collect();
        self.install_supporting_interface_types(supporting_interfaces)?;
        self.install_interfaces(interfaces)?;

        let previous_context = self.scopes.function_ctx.take();
        self.scopes.function_ctx = Some(FunctionCtx {
            name: unit.name.parts.join("."),
            return_type: None,
            owner_unit: Some(unit.name.parts.join(".")),
        });
        self.collect_unit_types(&unit.declarations);
        self.check_scope_declarations(&unit.declarations);
        self.scopes.function_ctx = previous_context;
        self.finish_pending_capture_checks();
        self.check_task_handle_escapes(&unit.declarations, None);
        self.validate_dictionary_keys();
        self.finish_contract_checks();
        self.finish_deferred_type_arguments();
        self.check_initialization(&unit.declarations, None);
        Ok(())
    }

    fn prepare_program(&mut self, program: &Program) {
        self.prepare_uses(&program.uses, &program.uses, None);
    }

    fn prepare_uses(
        &mut self,
        uses: &[fpas_parser::Import],
        intrinsic_uses: &[fpas_parser::Import],
        owner_root: Option<&str>,
    ) {
        self.register_primitive_types();
        self.initialize_imports(uses, owner_root);
        self.used_unit_names = uses
            .iter()
            .map(|used| used.unit.parts.join(".").to_ascii_lowercase())
            .collect();
        self.loaded_std_units.clear();
        self.imported_candidates.clear();
        for u in intrinsic_uses {
            match canonical_unit_from_uses_clause(&u.unit) {
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
        self.scopes.define(
            "ArrayPopResult",
            Symbol {
                constant: None,
                ty: crate::types::array_pop_result_declaration(),
                mutable: false,
                kind: SymbolKind::Type,
                capability: crate::TransferCapability::Transferable,
            },
        );
        for name in &["integer", "real", "boolean", "string"] {
            self.scopes.define(
                name,
                Symbol {
                    constant: None,
                    ty: Ty::Named(name.to_string()),
                    mutable: false,
                    kind: SymbolKind::Type,
                    capability: crate::TransferCapability::Transferable,
                },
            );
        }
    }

    /// Symbols from standard units that are actually in scope (requires matching `uses`).
    fn register_loaded_std_library(&mut self) {
        crate::std_registry::register_loaded_std(self);
        crate::std_registry::register_qualified_import_candidates(self);
    }
}
