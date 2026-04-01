use super::super::Checker;
use crate::types::{FunctionTy, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::SEMA_UNKNOWN_TYPE;
use fpas_parser::{QualifiedId, TypeExpr};

impl Checker {
    pub(crate) fn resolve_type_expr(&mut self, type_expr: &TypeExpr) -> Ty {
        match type_expr {
            TypeExpr::Named { id, .. } => self.resolve_named_type(id),
            TypeExpr::Array(inner, _) => Ty::Array(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::FunctionType {
                params,
                return_type,
                ..
            } => {
                let param_tys = params
                    .iter()
                    .map(|param| ParamTy {
                        mutable: param.mutable,
                        name: param.name.clone(),
                        ty: self.resolve_type_expr(&param.type_expr),
                    })
                    .collect();
                let return_ty = self.resolve_type_expr(return_type);
                Ty::Function(FunctionTy {
                    type_params: Vec::new(),
                    params: param_tys,
                    return_type: Box::new(return_ty),
                })
            }
            TypeExpr::ProcedureType { params, .. } => {
                let param_tys = params
                    .iter()
                    .map(|param| ParamTy {
                        mutable: param.mutable,
                        name: param.name.clone(),
                        ty: self.resolve_type_expr(&param.type_expr),
                    })
                    .collect();
                Ty::Procedure(ProcedureTy {
                    type_params: Vec::new(),
                    params: param_tys,
                    variadic: false,
                })
            }
            TypeExpr::Result {
                ok_type, err_type, ..
            } => {
                let ok = self.resolve_type_expr(ok_type);
                let err = self.resolve_type_expr(err_type);
                Ty::Result(Box::new(ok), Box::new(err))
            }
            TypeExpr::Option { inner_type, .. } => {
                Ty::Option(Box::new(self.resolve_type_expr(inner_type)))
            }
            TypeExpr::Dict {
                key_type,
                value_type,
                ..
            } => {
                let key = self.resolve_type_expr(key_type);
                let value = self.resolve_type_expr(value_type);
                Ty::Dict(Box::new(key), Box::new(value))
            }
        }
    }

    fn resolve_named_type(&mut self, qid: &QualifiedId) -> Ty {
        let name = qid.parts.join(".");
        match name.as_str() {
            "integer" => Ty::Integer,
            "real" => Ty::Real,
            "boolean" => Ty::Boolean,
            "char" => Ty::Char,
            "string" => Ty::String,
            "task" => Ty::Task(Box::new(Ty::Error)),
            _ => {
                if let Some(symbol) = self.scopes.lookup(&name) {
                    symbol.ty.clone()
                } else if self.ambiguous_hint(&name).is_some() {
                    self.report_ambiguous_type_name(&name, qid.span);
                    Ty::Error
                } else {
                    self.error_with_code(
                        SEMA_UNKNOWN_TYPE,
                        format!("Unknown type `{name}`"),
                        "Check spelling or add a type definition.",
                        qid.span,
                    );
                    Ty::Error
                }
            }
        }
    }

}
