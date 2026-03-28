mod calls;
mod designator;
mod operators;

use super::Checker;
use crate::types::Ty;
use fpas_parser::*;

impl Checker {
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Ty {
        let ty = match expr {
            Expr::Integer(_, _) => Ty::Integer,
            Expr::Real(_, _) => Ty::Real,
            Expr::Str(value, _) => {
                if value.chars().count() == 1 {
                    Ty::Char
                } else {
                    Ty::String
                }
            }
            Expr::Bool(_, _) => Ty::Boolean,
            Expr::Designator(designator) => self.check_designator_expr(designator),
            Expr::Call {
                designator,
                args,
                span,
            } => self.check_call_expr(expr, designator, args, *span),
            Expr::UnaryOp { op, operand, span } => self.check_unary_expr(*op, operand, *span),
            Expr::BinaryOp {
                op,
                left,
                right,
                span,
            } => self.check_binary_expr(*op, left, right, *span),
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::ArrayLiteral(elements, _) => self.check_array_literal(elements),
            Expr::DictLiteral(pairs, _) => self.check_dict_literal(pairs),
            Expr::RecordLiteral { fields, .. } => self.check_record_literal(fields),
            Expr::ResultOk(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Result(Box::new(inner_ty), Box::new(Ty::Error))
            }
            Expr::ResultError(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Result(Box::new(Ty::Error), Box::new(inner_ty))
            }
            Expr::OptionSome(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Option(Box::new(inner_ty))
            }
            Expr::OptionNone(_) => Ty::Option(Box::new(Ty::Error)),
            Expr::Try(inner, span) => self.check_try_expr(inner, *span),
            Expr::Function {
                params,
                return_type,
                body,
                span: _,
            } => self.check_function_expr(params, return_type, body),
            Expr::Go(inner, _) => {
                let inner_ty = self.check_expr(inner);
                Ty::Task(Box::new(inner_ty))
            }
        };
        let key = Self::expr_lookup_key(expr);
        self.expr_types.insert(key, ty.clone());
        ty
    }

    fn check_array_literal(&mut self, elements: &[Expr]) -> Ty {
        if elements.is_empty() {
            return Ty::Array(Box::new(Ty::Error));
        }

        let first_ty = self.check_expr(&elements[0]);
        for element in &elements[1..] {
            let element_ty = self.check_expr(element);
            self.check_type_compat(
                &first_ty,
                &element_ty,
                "array element",
                super::spans::expr_span(element),
            );
        }

        Ty::Array(Box::new(first_ty))
    }

    fn check_dict_literal(&mut self, pairs: &[(Expr, Expr)]) -> Ty {
        if pairs.is_empty() {
            return Ty::Dict(Box::new(Ty::Error), Box::new(Ty::Error));
        }

        let first_key_ty = self.check_expr(&pairs[0].0);
        let first_val_ty = self.check_expr(&pairs[0].1);
        for (key, val) in &pairs[1..] {
            let key_ty = self.check_expr(key);
            self.check_type_compat(
                &first_key_ty,
                &key_ty,
                "dict key",
                super::spans::expr_span(key),
            );
            let val_ty = self.check_expr(val);
            self.check_type_compat(
                &first_val_ty,
                &val_ty,
                "dict value",
                super::spans::expr_span(val),
            );
        }

        Ty::Dict(Box::new(first_key_ty), Box::new(first_val_ty))
    }

    fn check_record_literal(&mut self, fields: &[FieldInit]) -> Ty {
        let field_types = fields
            .iter()
            .map(|field| (field.name.clone(), self.check_expr(&field.value)))
            .collect();

        Ty::Record(crate::types::RecordTy {
            name: "<anonymous>".into(),
            type_params: Vec::new(),
            fields: field_types,
            methods: Vec::new(),
        })
    }

    fn check_try_expr(&mut self, inner: &Expr, span: fpas_lexer::Span) -> Ty {
        let inner_ty = self.check_expr(inner);
        match &inner_ty {
            Ty::Result(ok, _) => *ok.clone(),
            Ty::Option(inner) => *inner.clone(),
            Ty::Error => Ty::Error,
            _ => {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("try requires Result or Option, found `{inner_ty:?}`"),
                    "Use try only on Result or Option values.".to_string(),
                    span,
                );
                Ty::Error
            }
        }
    }

    /// Type-check an anonymous function expression (lambda / closure).
    ///
    /// **Documentation:** `docs/future/closures.md`
    fn check_function_expr(
        &mut self,
        params: &[FormalParam],
        return_type_expr: &TypeExpr,
        body: &FuncBody,
    ) -> Ty {
        use crate::scope::{FunctionCtx, Symbol, SymbolKind};
        use crate::types::{FunctionTy, ParamTy};

        let return_ty = self.resolve_type_expr(return_type_expr);
        let param_tys: Vec<ParamTy> = params
            .iter()
            .map(|p| ParamTy {
                mutable: p.mutable,
                name: p.name.clone(),
                ty: self.resolve_type_expr(&p.type_expr),
            })
            .collect();

        if let FuncBody::Block { nested, stmts } = body {
            self.scopes.push_scope();

            for p in &param_tys {
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
                name: "<lambda>".into(),
                return_type: Some(return_ty.clone()),
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

        Ty::Function(FunctionTy {
            params: param_tys,
            return_type: Box::new(return_ty),
        })
    }
}
