use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty};
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl};

impl Checker {
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

        if !self.scopes.define(
            &f.name,
            Symbol {
                ty: func_ty,
                mutable: false,
                kind: SymbolKind::Function,
            },
        ) {
            // Allow duplicate for forward + implementation pattern
        }

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

        if !self.scopes.define(
            &p.name,
            Symbol {
                ty: proc_ty,
                mutable: false,
                kind: SymbolKind::Procedure,
            },
        ) {
            // Allow duplicate for forward + implementation
        }

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

            self.scopes.function_ctx = prev_ctx;
            self.scopes.pop_scope();
        }
    }
}
