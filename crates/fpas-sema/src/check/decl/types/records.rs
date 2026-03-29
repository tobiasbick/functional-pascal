//! Record type checking.
//!
//! **Documentation:** `docs/pascal/05-types.md`

use super::Checker;
use crate::scope::{FunctionCtx, Symbol, SymbolKind};
use crate::types::{FunctionTy, MethodKind, ParamTy, ProcedureTy, RecordTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{FuncBody, RecordMethod, RecordType, TypeDef};

impl Checker {
    pub(super) fn check_record_type_def(&mut self, td: &TypeDef, record: &RecordType) {
        let fields = self.with_type_params(&td.type_params, td.span, |checker| {
            record
                .fields
                .iter()
                .map(|field| {
                    (
                        field.name.clone(),
                        checker.resolve_type_expr(&field.type_expr),
                    )
                })
                .collect::<Vec<_>>()
        });

        let record_ty = RecordTy {
            name: td.name.clone(),
            type_params: Self::resolve_type_params(&td.type_params),
            fields,
            methods: Vec::new(),
        };
        let mut ty = Ty::Record(record_ty);

        if !self.define_type_symbol(td, ty.clone()) {
            return;
        }

        let methods = self.with_type_params(&td.type_params, td.span, |checker| {
            checker.check_record_methods(&td.name, &ty, &record.methods)
        });

        if let Ty::Record(record_ty) = &mut ty {
            record_ty.methods = methods;
        }

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing.ty_mut() = ty;
        }
    }

    fn check_record_methods(
        &mut self,
        type_name: &str,
        record_ty: &Ty,
        methods: &[RecordMethod],
    ) -> Vec<(String, MethodKind)> {
        let mut checked_methods = Vec::new();

        for method in methods {
            match method {
                RecordMethod::Function(function) => {
                    let return_ty = self.resolve_type_expr(&function.return_type);
                    let params: Vec<ParamTy> = function
                        .params
                        .iter()
                        .map(|param| ParamTy {
                            mutable: param.mutable,
                            name: param.name.clone(),
                            ty: self.resolve_method_param_type(
                                &param.type_expr,
                                type_name,
                                record_ty,
                            ),
                        })
                        .collect();

                    if !self.validate_record_method_signature(
                        type_name,
                        &function.name,
                        &params,
                        function.span,
                    ) {
                        continue;
                    }

                    let function_ty = FunctionTy {
                        type_params: Vec::new(),
                        params: params.clone(),
                        return_type: Box::new(return_ty.clone()),
                    };

                    let qualified = format!("{type_name}.{}", function.name);
                    self.scopes.define(
                        &qualified,
                        Symbol {
                            ty: Ty::Function(function_ty.clone()),
                            mutable: false,
                            kind: SymbolKind::Function,
                        },
                    );

                    self.check_method_body(&qualified, &params, Some(return_ty), &function.body);
                    checked_methods
                        .push((function.name.clone(), MethodKind::Function(function_ty)));
                }
                RecordMethod::Procedure(procedure) => {
                    let params: Vec<ParamTy> = procedure
                        .params
                        .iter()
                        .map(|param| ParamTy {
                            mutable: param.mutable,
                            name: param.name.clone(),
                            ty: self.resolve_method_param_type(
                                &param.type_expr,
                                type_name,
                                record_ty,
                            ),
                        })
                        .collect();

                    if !self.validate_record_method_signature(
                        type_name,
                        &procedure.name,
                        &params,
                        procedure.span,
                    ) {
                        continue;
                    }

                    let procedure_ty = ProcedureTy {
                        type_params: Vec::new(),
                        variadic: false,
                        params: params.clone(),
                    };

                    let qualified = format!("{type_name}.{}", procedure.name);
                    self.scopes.define(
                        &qualified,
                        Symbol {
                            ty: Ty::Procedure(procedure_ty.clone()),
                            mutable: false,
                            kind: SymbolKind::Procedure,
                        },
                    );

                    self.check_method_body(&qualified, &params, None, &procedure.body);
                    checked_methods
                        .push((procedure.name.clone(), MethodKind::Procedure(procedure_ty)));
                }
            }
        }

        checked_methods
    }

    fn validate_record_method_signature(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: &[ParamTy],
        span: fpas_lexer::Span,
    ) -> bool {
        let Some(self_param) = params.first() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        };

        if !self_param.name.eq_ignore_ascii_case("Self")
            || !matches!(&self_param.ty, Ty::Record(record) if record.name.eq_ignore_ascii_case(type_name))
        {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "Record method `{type_name}.{method_name}` must declare `Self: {type_name}` as its first parameter"
                ),
                format!(
                    "Use `{method_name}(Self: {type_name}; ...)` so calls like `Value.{method_name}(...)` can pass the receiver implicitly."
                ),
                span,
            );
            return false;
        }

        true
    }

    fn check_method_body(
        &mut self,
        qualified_name: &str,
        params: &[ParamTy],
        return_type: Option<Ty>,
        body: &FuncBody,
    ) {
        let FuncBody::Block { nested, stmts } = body else {
            return;
        };

        self.scopes.push_scope();
        for param in params {
            self.scopes.define(
                &param.name,
                Symbol {
                    ty: param.ty.clone(),
                    mutable: param.mutable,
                    kind: SymbolKind::Param,
                },
            );
        }
        let previous_ctx = self.scopes.function_ctx.take();
        self.scopes.function_ctx = Some(FunctionCtx {
            name: qualified_name.to_string(),
            return_type,
        });
        for decl in nested {
            self.check_decl(decl);
        }
        for stmt in stmts {
            self.check_stmt(stmt);
        }
        self.report_missing_forward_declarations_in_current_scope();
        self.scopes.function_ctx = previous_ctx;
        self.scopes.pop_scope();
    }

    /// Resolve a method parameter type, treating the type name as the record type (for `Self`).
    fn resolve_method_param_type(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
        type_name: &str,
        record_ty: &Ty,
    ) -> Ty {
        let resolved = self.resolve_type_expr(type_expr);
        if let Ty::Named(name) = &resolved
            && name == type_name
        {
            return record_ty.clone();
        }
        resolved
    }
}
