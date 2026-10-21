//! Routine declaration checking, including nested capture analysis.
//!
//! **Documentation:** `docs/pascal/language/functions/README.md`,
//! `docs/pascal/language/functions/closures.md`

use super::super::closures::{
    CaptureAnalysis, CaptureOwner, NestedRoutineCaptureInfo, capability_from_captures,
};
use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_lexer::Span;
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl};

impl Checker {
    /// Check function declarations against `docs/pascal/language/functions/README.md`.
    pub(super) fn check_function_decl(&mut self, f: &FunctionDecl) {
        if !self.has_routine_signature(&f.name, f.span) {
            self.collect_function_signature(f);
        }
        let Some(Ty::Function(function)) = self.scopes.lookup(&f.name).map(|s| s.ty.clone()) else {
            return;
        };
        let is_nested = self.scopes.scope_count() > 1;
        let captures = self.check_routine_body_collecting_captures(
            &f.name,
            &f.type_params,
            &function.params,
            &f.params,
            Some(*function.return_type),
            super::super::contracts::RoutineBody {
                body: &f.body,
                requires: &f.requires,
                ensures: &f.ensures,
            },
        );
        if is_nested {
            self.record_nested_routine_captures(
                &f.name,
                crate::function_decl_lookup_key(f),
                captures,
            );
        }
    }

    /// Resolve and install a named function before checking any sibling body.
    pub(super) fn collect_function_signature(&mut self, f: &FunctionDecl) {
        self.check_unique_formal_param_names(&f.params);

        let has_type_params = !f.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&f.type_params, f.span);
        }

        let type_param_defs = self.routine_type_params(&f.name, &f.type_params);
        let return_ty = self.resolve_type_expr(&f.return_type);
        let params: Vec<ParamTy> = self.resolve_formal_params(&f.params);

        let func_ty = Ty::Function(FunctionTy {
            type_params: type_param_defs,
            params: params.clone(),
            return_type: Box::new(return_ty.clone()),
            variadic: false,
        });

        if has_type_params {
            self.scopes.pop_scope();
        }

        let is_nested = self.scopes.scope_count() > 1;
        let symbol = Symbol {
            constant: None,
            ty: func_ty,
            mutable: false,
            kind: SymbolKind::Function,
            capability: crate::TransferCapability::Transferable,
        };
        self.register_routine_symbol(&f.name, symbol, &f.body, f.span);
        if is_nested {
            self.scopes
                .set_routine_capture_key(&f.name, crate::function_decl_lookup_key(f));
        }
    }

    /// Check procedure declarations against `docs/pascal/language/functions/README.md`.
    pub(super) fn check_procedure_decl(&mut self, p: &ProcedureDecl) {
        if !self.has_routine_signature(&p.name, p.span) {
            self.collect_procedure_signature(p);
        }
        let Some(Ty::Procedure(procedure)) = self.scopes.lookup(&p.name).map(|s| s.ty.clone())
        else {
            return;
        };
        let is_nested = self.scopes.scope_count() > 1;
        let captures = self.check_routine_body_collecting_captures(
            &p.name,
            &p.type_params,
            &procedure.params,
            &p.params,
            None,
            super::super::contracts::RoutineBody {
                body: &p.body,
                requires: &p.requires,
                ensures: &p.ensures,
            },
        );
        if is_nested {
            self.record_nested_routine_captures(
                &p.name,
                crate::procedure_decl_lookup_key(p),
                captures,
            );
        }
    }

    /// Resolve and install a named procedure before checking any sibling body.
    pub(super) fn collect_procedure_signature(&mut self, p: &ProcedureDecl) {
        self.check_unique_formal_param_names(&p.params);

        let has_type_params = !p.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&p.type_params, p.span);
        }

        let type_param_defs = self.routine_type_params(&p.name, &p.type_params);
        let params: Vec<ParamTy> = self.resolve_formal_params(&p.params);

        let proc_ty = Ty::Procedure(ProcedureTy {
            type_params: type_param_defs,
            variadic: false,
            params: params.clone(),
        });

        if has_type_params {
            self.scopes.pop_scope();
        }

        let is_nested = self.scopes.scope_count() > 1;
        let symbol = Symbol {
            constant: None,
            ty: proc_ty,
            mutable: false,
            kind: SymbolKind::Procedure,
            capability: crate::TransferCapability::Transferable,
        };
        self.register_routine_symbol(&p.name, symbol, &p.body, p.span);
        if is_nested {
            self.scopes
                .set_routine_capture_key(&p.name, crate::procedure_decl_lookup_key(p));
        }
    }

    pub(crate) fn resolve_formal_params(
        &mut self,
        params: &[fpas_parser::FormalParam],
    ) -> Vec<ParamTy> {
        params
            .iter()
            .map(|p| ParamTy {
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
                mode: p.mode,
            })
            .collect()
    }

    fn record_nested_routine_captures(
        &mut self,
        name: &str,
        key: usize,
        analysis: CaptureAnalysis,
    ) {
        self.register_capture_dependencies(
            CaptureOwner::Routine(key),
            analysis.scope_index,
            &analysis.pending_routines,
        );
        self.pending_routine_captures
            .insert(key, analysis.pending_routines);
        let captures = analysis.captures;
        let reference = captures
            .iter()
            .find(|capture| capture.reference)
            .map(|capture| capture.name.clone());
        let capability = capability_from_captures(&captures);
        if let Some(symbol) = self.scopes.lookup_mut(name) {
            symbol.capability = capability;
        }
        self.nested_routine_captures.insert(
            key,
            NestedRoutineCaptureInfo {
                captures,
                capability,
            },
        );
        self.record_var_parameter_routine(key, reference);
        self.propagate_pending_captures(key);
    }

    fn register_routine_symbol(&mut self, name: &str, symbol: Symbol, body: &FuncBody, span: Span) {
        match body {
            FuncBody::Block { .. } => match self.install_routine_symbol(name, symbol, span) {
                RoutineInstall::Installed => {}
                RoutineInstall::Duplicate => {
                    self.error_with_code(
                        SEMA_DUPLICATE_DECLARATION,
                        format!("Duplicate routine `{name}`"),
                        "Each routine name must be unique in the same scope.",
                        span,
                    );
                }
            },
        }
    }

    fn install_routine_symbol(&mut self, name: &str, symbol: Symbol, span: Span) -> RoutineInstall {
        if self.scopes.define_with_declaration(name, symbol, span) {
            return RoutineInstall::Installed;
        }

        RoutineInstall::Duplicate
    }
}

enum RoutineInstall {
    Installed,
    Duplicate,
}
