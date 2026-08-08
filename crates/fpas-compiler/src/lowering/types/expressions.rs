//! Parser type-expression resolution for the register type table.

use std::collections::BTreeSet;

use fpas_ir::{IrType, TypeId};

use crate::CompileError;

use super::{BOOLEAN, DYNAMIC, INTEGER, REAL, STRING, TypeTable, UNIT};

impl TypeTable {
    pub fn type_expr(&mut self, type_expr: &fpas_parser::TypeExpr) -> Result<TypeId, CompileError> {
        self.type_expr_with_generics(type_expr, &BTreeSet::new())
    }

    pub fn type_expr_with_params(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
        type_params: &[fpas_parser::TypeParam],
    ) -> Result<TypeId, CompileError> {
        let generics = type_params
            .iter()
            .map(|parameter| parameter.name.to_ascii_lowercase())
            .collect();
        self.type_expr_with_generics(type_expr, &generics)
    }

    fn type_expr_with_generics(
        &mut self,
        type_expr: &fpas_parser::TypeExpr,
        generics: &BTreeSet<String>,
    ) -> Result<TypeId, CompileError> {
        use fpas_parser::TypeExpr;
        match type_expr {
            TypeExpr::Named { id, span } => {
                let name = id.parts.join(".");
                if id.parts.len() == 1 && generics.contains(&name.to_ascii_lowercase()) {
                    return Ok(DYNAMIC);
                }
                if let Some(id) = self.named.get(&name.to_ascii_lowercase()) {
                    return Ok(*id);
                }
                match name.to_ascii_lowercase().as_str() {
                    "integer" => Ok(INTEGER),
                    "real" => Ok(REAL),
                    "boolean" => Ok(BOOLEAN),
                    "string" => Ok(STRING),
                    "task" => self.intern_kind(IrType::Task(DYNAMIC), *span),
                    _ => {
                        if let Some(layout) = self
                            .record_layouts
                            .iter()
                            .find(|layout| super::super::type_names::matches(&layout.name, &name))
                        {
                            return self.intern_kind(IrType::Record(layout.id), *span);
                        }
                        if let Some(layout) = self
                            .enum_layouts
                            .iter()
                            .find(|layout| super::super::type_names::matches(&layout.name, &name))
                        {
                            return self.intern_kind(IrType::Enum(layout.id), *span);
                        }
                        if self.simple_enums.iter().any(|enumeration| {
                            super::super::type_names::matches(enumeration, &name)
                        }) {
                            return Ok(INTEGER);
                        }
                        Ok(DYNAMIC)
                    }
                }
            }
            TypeExpr::Array(element, span) => {
                let element = self.type_expr_with_generics(element, generics)?;
                self.intern_kind(IrType::Array(element), *span)
            }
            TypeExpr::Dict {
                key_type,
                value_type,
                span,
            } => {
                let key = self.type_expr_with_generics(key_type, generics)?;
                let value = self.type_expr_with_generics(value_type, generics)?;
                self.intern_kind(IrType::Dictionary { key, value }, *span)
            }
            TypeExpr::Result {
                ok_type,
                err_type,
                span,
            } => {
                let ok = self.type_expr_with_generics(ok_type, generics)?;
                let error = self.type_expr_with_generics(err_type, generics)?;
                self.intern_kind(IrType::Result { ok, error }, *span)
            }
            TypeExpr::Option { inner_type, span } => {
                let inner = self.type_expr_with_generics(inner_type, generics)?;
                self.intern_kind(IrType::Option(inner), *span)
            }
            TypeExpr::FunctionType {
                params,
                return_type,
                span,
            } => {
                let parameters = params
                    .iter()
                    .map(|parameter| self.type_expr_with_generics(&parameter.type_expr, generics))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.type_expr_with_generics(return_type, generics)?;
                self.intern_kind(IrType::Function { parameters, result }, *span)
            }
            TypeExpr::ProcedureType { params, span } => {
                let parameters = params
                    .iter()
                    .map(|parameter| self.type_expr_with_generics(&parameter.type_expr, generics))
                    .collect::<Result<Vec<_>, _>>()?;
                self.intern_kind(
                    IrType::Function {
                        parameters,
                        result: UNIT,
                    },
                    *span,
                )
            }
        }
    }
}
