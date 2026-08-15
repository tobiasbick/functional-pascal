//! Routine declaration checking, including nested capture analysis.
//!
//! **Documentation:** `docs/pascal/language/functions/README.md`,
//! `docs/pascal/language/functions/closures.md`

use super::super::closures::{
    CaptureBinding, NestedRoutineCaptureInfo, collect_captures, task_bound_from_captures,
};
use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty, TypeConstraint};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_lexer::Span;
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl, Stmt};

impl Checker {
    /// Check function declarations against `docs/pascal/language/functions/README.md`.
    pub(super) fn check_function_decl(&mut self, f: &FunctionDecl) {
        self.check_unique_formal_param_names(&f.params);

        let has_type_params = !f.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&f.type_params, f.span);
        }

        let type_param_defs = Self::resolve_type_params(&f.type_params);
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
            ty: func_ty,
            mutable: false,
            kind: SymbolKind::Function,
            task_bound: false,
        };
        self.register_routine_symbol(&f.name, symbol, &f.body, f.span);
        let captures = self.check_routine_body_collecting_captures(
            &f.name,
            &f.type_params,
            &params,
            &f.params,
            Some(return_ty),
            &f.body,
        );
        if is_nested {
            self.record_nested_routine_captures(
                &f.name,
                crate::function_decl_lookup_key(f),
                captures,
            );
        }
    }

    /// Check procedure declarations against `docs/pascal/language/functions/README.md`.
    pub(super) fn check_procedure_decl(&mut self, p: &ProcedureDecl) {
        self.check_unique_formal_param_names(&p.params);

        let has_type_params = !p.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&p.type_params, p.span);
        }

        let type_param_defs = Self::resolve_type_params(&p.type_params);
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
            ty: proc_ty,
            mutable: false,
            kind: SymbolKind::Procedure,
            task_bound: false,
        };
        self.register_routine_symbol(&p.name, symbol, &p.body, p.span);
        let captures = self.check_routine_body_collecting_captures(
            &p.name,
            &p.type_params,
            &params,
            &p.params,
            None,
            &p.body,
        );
        if is_nested {
            self.record_nested_routine_captures(
                &p.name,
                crate::procedure_decl_lookup_key(p),
                captures,
            );
        }
    }

    pub(crate) fn resolve_formal_params(
        &mut self,
        params: &[fpas_parser::FormalParam],
    ) -> Vec<ParamTy> {
        params
            .iter()
            .map(|p| ParamTy {
                mutable: p.mutable,
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
            })
            .collect()
    }

    /// Check a routine body and return lexical captures of the routine itself.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    pub(crate) fn check_routine_body_collecting_captures(
        &mut self,
        name: &str,
        type_params: &[fpas_parser::TypeParam],
        params: &[ParamTy],
        source_params: &[fpas_parser::FormalParam],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) -> Vec<CaptureBinding> {
        let FuncBody::Block { nested, stmts } = body;

        self.scopes.push_scope();
        let routine_scope_index = self.scopes.scope_count() - 1;

        for tp in type_params {
            let constraint = tp
                .constraint
                .as_ref()
                .and_then(|c| TypeConstraint::from_name(c));
            self.scopes.define(
                &tp.name,
                Symbol {
                    ty: Ty::GenericParam(tp.name.clone(), constraint),
                    mutable: false,
                    kind: SymbolKind::Type,
                    task_bound: false,
                },
            );
        }

        for (p, source) in params.iter().zip(source_params) {
            self.scopes.define_with_declaration(
                &p.name,
                Symbol {
                    ty: p.ty.clone(),
                    mutable: p.mutable,
                    kind: SymbolKind::Param,
                    task_bound: false,
                },
                source.span,
            );
        }

        let prev_ctx = self.scopes.function_ctx.take();
        let owner_unit = if let Some(context) = &prev_ctx {
            context.owner_unit.clone()
        } else {
            name.rsplit_once('.').map(|(unit, _)| unit.to_string())
        };
        self.scopes.function_ctx = Some(FunctionCtx {
            name: name.to_string(),
            return_type,
            owner_unit,
        });

        let hoisted = self.hoist_body_locals(stmts);
        for decl in nested {
            self.check_decl(decl);
        }
        for name in &hoisted {
            self.scopes.remove_from_current(name);
        }
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        let captures = collect_captures(
            &self.scopes,
            routine_scope_index,
            body,
            &self.closure_infos,
            &self.nested_routine_captures,
        );

        self.scopes.function_ctx = prev_ctx;
        self.scopes.pop_scope();
        captures
    }

    /// Make function-body locals visible to nested routine bodies without changing
    /// sequential sibling visibility in the enclosing body.
    ///
    /// Nested declarations are checked before body statements, but
    /// `docs/pascal/language/functions/closures.md` lets named nested routines capture
    /// enclosing locals. Locals nested inside inner `begin` blocks stay hidden.
    fn hoist_body_locals(&mut self, stmts: &[Stmt]) -> Vec<String> {
        let mut names = Vec::new();
        for stmt in stmts {
            let (variable, mutable) = match stmt {
                Stmt::Var(variable) => (variable, false),
                Stmt::MutableVar(variable) => (variable, true),
                _ => continue,
            };
            let ty = self.resolve_type_expr(&variable.type_expr);
            let defined = self.scopes.define_with_declaration(
                &variable.name,
                Symbol {
                    ty,
                    mutable,
                    kind: SymbolKind::Var,
                    task_bound: false,
                },
                variable.span,
            );
            if defined {
                names.push(variable.name.clone());
            }
        }
        names
    }

    fn record_nested_routine_captures(
        &mut self,
        name: &str,
        key: usize,
        captures: Vec<CaptureBinding>,
    ) {
        let task_bound = task_bound_from_captures(&captures);
        if let Some(symbol) = self.scopes.lookup_mut(name) {
            symbol.task_bound = task_bound;
        }
        self.nested_routine_captures.insert(
            key,
            NestedRoutineCaptureInfo {
                captures,
                task_bound,
            },
        );
    }

    fn register_routine_symbol(&mut self, name: &str, symbol: Symbol, body: &FuncBody, span: Span) {
        match body {
            FuncBody::Block { .. } => match self.install_routine_symbol(name, symbol) {
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

    fn install_routine_symbol(&mut self, name: &str, symbol: Symbol) -> RoutineInstall {
        if self.scopes.define(name, symbol.clone()) {
            return RoutineInstall::Installed;
        }

        let Some(existing) = self.scopes.lookup_current(name) else {
            return RoutineInstall::Duplicate;
        };

        if existing.kind != SymbolKind::BuiltinStd {
            return RoutineInstall::Duplicate;
        }

        if let Some(existing) = self.scopes.lookup_mut(name) {
            *existing = symbol;
            return RoutineInstall::Installed;
        }

        RoutineInstall::Duplicate
    }
}

enum RoutineInstall {
    Installed,
    Duplicate,
}
