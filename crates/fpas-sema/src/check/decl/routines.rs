use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty, TypeConstraint};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_lexer::Span;
use fpas_parser::{FuncBody, FunctionDecl, ProcedureDecl};

impl Checker {
    /// Check function declarations against `docs/pascal/04-functions.md`.
    pub(super) fn check_function_decl(&mut self, f: &FunctionDecl) {
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
        });

        if has_type_params {
            self.scopes.pop_scope();
        }

        let symbol = Symbol {
            ty: func_ty,
            mutable: false,
            kind: SymbolKind::Function,
        };
        self.register_routine_symbol(&f.name, symbol, &f.body, f.span);
        self.check_routine_body(&f.name, &f.type_params, &params, Some(return_ty), &f.body);
    }

    /// Check procedure declarations against `docs/pascal/04-functions.md`.
    pub(super) fn check_procedure_decl(&mut self, p: &ProcedureDecl) {
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

        let symbol = Symbol {
            ty: proc_ty,
            mutable: false,
            kind: SymbolKind::Procedure,
        };
        self.register_routine_symbol(&p.name, symbol, &p.body, p.span);
        self.check_routine_body(&p.name, &p.type_params, &params, None, &p.body);
    }

    fn resolve_formal_params(&mut self, params: &[fpas_parser::FormalParam]) -> Vec<ParamTy> {
        params
            .iter()
            .map(|p| ParamTy {
                mutable: p.mutable,
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
            })
            .collect()
    }

    fn check_routine_body(
        &mut self,
        name: &str,
        type_params: &[fpas_parser::TypeParam],
        params: &[ParamTy],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) {
        let FuncBody::Block { nested, stmts } = body else {
            return;
        };

        self.scopes.push_scope();

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
                },
            );
        }

        for p in params {
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
            name: name.to_string(),
            return_type,
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

    fn register_routine_symbol(&mut self, name: &str, symbol: Symbol, body: &FuncBody, span: Span) {
        match body {
            FuncBody::SignatureOnly => {}
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
