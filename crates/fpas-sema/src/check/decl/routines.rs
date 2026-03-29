use super::Checker;
use crate::scope::{FunctionCtx, PendingRoutine, Symbol, SymbolKind};
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl};

impl Checker {
    /// Check function and procedure declarations against `docs/pascal/04-functions.md`.
    pub(super) fn check_function_decl(&mut self, f: &FunctionDecl) {
        // Register generic type parameters as GenericParam types in a new scope.
        let has_type_params = !f.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&f.type_params, f.span);
        }

        let return_ty = self.resolve_type_expr(&f.return_type);
        let params: Vec<ParamTy> = f
            .params
            .iter()
            .map(|p| ParamTy {
                mutable: p.mutable,
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
            })
            .collect();

        let func_ty = Ty::Function(FunctionTy {
            params: params.clone(),
            return_type: Box::new(return_ty.clone()),
        });

        if has_type_params {
            self.scopes.pop_scope();
        }

        self.register_routine_symbol(
            &f.name,
            Symbol {
                ty: func_ty,
                mutable: false,
                kind: SymbolKind::Function,
            },
            &f.body,
            f.span,
        );

        if let FuncBody::Block { nested, stmts } = &f.body {
            self.scopes.push_scope();

            // Re-introduce type params inside the function body scope.
            for tp in &f.type_params {
                self.scopes.define(
                    &tp.name,
                    Symbol {
                        ty: Ty::GenericParam(tp.name.clone()),
                        mutable: false,
                        kind: SymbolKind::Type,
                    },
                );
            }

            for p in &params {
                self.scopes.define(
                    &p.name,
                    Symbol {
                        ty: p.ty.clone(),
                        mutable: p.mutable,
                        kind: SymbolKind::Param,
                    },
                );
            }

            let prev_ctx = self.scopes.function_ctx.take();
            self.scopes.function_ctx = Some(FunctionCtx {
                name: f.name.clone(),
                return_type: Some(return_ty),
            });

            for decl in nested {
                self.check_decl(decl);
            }

            for stmt in stmts {
                self.check_stmt(stmt);
            }

            self.report_missing_forward_declarations_in_current_scope();
            self.scopes.function_ctx = prev_ctx;
            self.scopes.pop_scope();
        }
    }

    pub(super) fn check_procedure_decl(&mut self, p: &ProcedureDecl) {
        // Register generic type parameters.
        let has_type_params = !p.type_params.is_empty();
        if has_type_params {
            self.push_type_param_scope(&p.type_params, p.span);
        }

        let params: Vec<ParamTy> = p
            .params
            .iter()
            .map(|param| ParamTy {
                mutable: param.mutable,
                name: param.name.clone(),
                ty: self.resolve_type_expr(&param.type_expr),
            })
            .collect();

        let proc_ty = Ty::Procedure(ProcedureTy {
            variadic: false,
            params: params.clone(),
        });

        if has_type_params {
            self.scopes.pop_scope();
        }

        self.register_routine_symbol(
            &p.name,
            Symbol {
                ty: proc_ty,
                mutable: false,
                kind: SymbolKind::Procedure,
            },
            &p.body,
            p.span,
        );

        if let FuncBody::Block { nested, stmts } = &p.body {
            self.scopes.push_scope();

            // Re-introduce type params inside the procedure body scope.
            for tp in &p.type_params {
                self.scopes.define(
                    &tp.name,
                    Symbol {
                        ty: Ty::GenericParam(tp.name.clone()),
                        mutable: false,
                        kind: SymbolKind::Type,
                    },
                );
            }

            for param in &params {
                self.scopes.define(
                    &param.name,
                    Symbol {
                        ty: param.ty.clone(),
                        mutable: param.mutable,
                        kind: SymbolKind::Param,
                    },
                );
            }

            let prev_ctx = self.scopes.function_ctx.take();
            self.scopes.function_ctx = Some(FunctionCtx {
                name: p.name.clone(),
                return_type: None,
            });

            for decl in nested {
                self.check_decl(decl);
            }

            for stmt in stmts {
                self.check_stmt(stmt);
            }

            self.report_missing_forward_declarations_in_current_scope();
            self.scopes.function_ctx = prev_ctx;
            self.scopes.pop_scope();
        }
    }

    fn register_routine_symbol(&mut self, name: &str, symbol: Symbol, body: &FuncBody, span: Span) {
        match body {
            FuncBody::Forward => self.register_forward_routine(name, symbol, span),
            FuncBody::Block { .. } => self.register_routine_implementation(name, symbol, span),
        }
    }

    fn register_forward_routine(&mut self, name: &str, symbol: Symbol, span: Span) {
        match self.install_routine_symbol(name, symbol.clone()) {
            RoutineInstall::Installed => {
                self.scopes
                    .define_pending_routine(name, PendingRoutine { symbol, span });
            }
            RoutineInstall::Duplicate => {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate routine `{name}`"),
                    "Use exactly one forward declaration followed by one matching implementation.",
                    span,
                );
            }
        }
    }

    fn register_routine_implementation(&mut self, name: &str, symbol: Symbol, span: Span) {
        match self.install_routine_symbol(name, symbol.clone()) {
            RoutineInstall::Installed => return,
            RoutineInstall::Duplicate => {}
        }

        let Some(forward) = self.scopes.take_pending_routine(name) else {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate routine `{name}`"),
                "Each routine name must be unique in the same scope.",
                span,
            );
            return;
        };

        if forward.symbol.kind != symbol.kind || forward.symbol.ty != symbol.ty {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Forward declaration for `{name}` does not match its implementation"),
                "Make the implementation use the same parameters, routine kind, and return type as the forward declaration.",
                span,
            );
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

    pub(crate) fn report_missing_forward_declarations_in_current_scope(&mut self) {
        for (name, pending) in self.scopes.drain_pending_routines() {
            let routine_kind = match pending.symbol.kind {
                SymbolKind::Function => "Function",
                SymbolKind::Procedure => "Procedure",
                _ => "Routine",
            };
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!("{routine_kind} `{name}` was declared `forward` but never implemented"),
                "Add a matching body later in the same declaration scope.",
                pending.span,
            );
        }
    }
}

enum RoutineInstall {
    Installed,
    Duplicate,
}
