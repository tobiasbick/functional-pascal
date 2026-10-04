//! Prepare initializer types for named routines that capture enclosing body locals.

use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{FunctionTy, ProcedureTy, Ty};
use fpas_parser::{Decl, Stmt};

impl Checker {
    /// Expose immediate body locals to nested routines without inferring signatures.
    pub(in crate::check) fn hoist_body_locals(
        &mut self,
        stmts: &[Stmt],
        nested: &[Decl],
    ) -> Vec<String> {
        if nested.is_empty() {
            return Vec::new();
        }
        let diagnostic_count = self.errors.len();
        let alias_conflict_count = self.scopes.import_alias_conflicts.len();
        self.scopes.push_scope();
        for declaration in nested {
            self.install_inference_header(declaration);
        }
        let mut bindings = Vec::new();
        for statement in stmts {
            let (definition, mutable) = match statement {
                Stmt::Const(definition) => (definition, false),
                Stmt::Var(definition) => (definition, true),
                _ => continue,
            };
            self.check_binding(definition, mutable);
            if let Some(symbol) = self.scopes.lookup(&definition.name) {
                bindings.push((definition, symbol.clone()));
            }
        }
        self.scopes.pop_scope();
        // The sequential body check reports initializer and alias errors once.
        self.errors.truncate(diagnostic_count);
        self.scopes
            .import_alias_conflicts
            .truncate(alias_conflict_count);
        let mut names = Vec::new();
        for (definition, symbol) in bindings {
            if self
                .scopes
                .define_with_declaration(&definition.name, symbol, definition.span)
            {
                names.push(definition.name.clone());
            }
        }
        self.scopes
            .import_alias_conflicts
            .truncate(alias_conflict_count);
        names
    }

    fn install_inference_header(&mut self, declaration: &Decl) {
        let (name, parameters, result, type_parameters, span) = match declaration {
            Decl::Function(routine) => (
                &routine.name,
                &routine.params,
                Some(&routine.return_type),
                &routine.type_params,
                routine.span,
            ),
            Decl::Procedure(routine) => (
                &routine.name,
                &routine.params,
                None,
                &routine.type_params,
                routine.span,
            ),
            _ => return,
        };
        if !type_parameters.is_empty() {
            self.push_type_param_scope(type_parameters, span);
        }
        let previous_proofs = self.add_pure_parameter_proofs(type_parameters, true);
        let params = self.resolve_formal_params(parameters);
        let type_params = self.resolve_type_params(type_parameters);
        let ty = if let Some(result) = result {
            Ty::Function(FunctionTy {
                pure: matches!(declaration, Decl::Function(routine) if routine.pure),
                type_params,
                params,
                return_type: Box::new(self.resolve_type_expr(result)),
                variadic: false,
            })
        } else {
            Ty::Procedure(ProcedureTy {
                type_params,
                params,
                variadic: false,
            })
        };
        self.pure_parameters = previous_proofs;
        if !type_parameters.is_empty() {
            self.scopes.pop_scope();
        }
        self.scopes.define(
            name,
            Symbol {
                ty,
                mutable: false,
                kind: if result.is_some() {
                    SymbolKind::Function
                } else {
                    SymbolKind::Procedure
                },
                task_bound: false,
            },
        );
    }
}
