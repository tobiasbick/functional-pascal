//! Compact semantic-to-IR scalar type mapping.

use fpas_ir::{IrType, TypeDefinition, TypeId};
use fpas_sema::Ty;

use crate::CompileError;
use crate::error::internal_compiler_error;

pub(super) const UNIT: TypeId = TypeId::new(0);
pub(super) const BOOLEAN: TypeId = TypeId::new(1);
pub(super) const INTEGER: TypeId = TypeId::new(2);
pub(super) const REAL: TypeId = TypeId::new(3);
pub(super) const STRING: TypeId = TypeId::new(4);
pub(super) const DYNAMIC: TypeId = TypeId::new(5);

#[derive(Debug, Clone)]
pub(super) struct TypeTable {
    definitions: Vec<TypeDefinition>,
}

impl TypeTable {
    pub fn from_metadata(metadata: &fpas_sema::AnalysisMetadata) -> Result<Self, CompileError> {
        let mut table = Self {
            definitions: scalar_type_table(),
        };
        for ty in metadata.expr_types.values() {
            let _ = table.intern(ty, 1, 1)?;
        }
        Ok(table)
    }

    pub fn intern(&mut self, ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
        let kind = match ty {
            Ty::Function(function) => IrType::Function {
                parameters: function
                    .params
                    .iter()
                    .map(|parameter| self.intern(&parameter.ty, line, column))
                    .collect::<Result<Vec<_>, _>>()?,
                result: self.intern(&function.return_type, line, column)?,
            },
            Ty::Procedure(procedure) => IrType::Function {
                parameters: procedure
                    .params
                    .iter()
                    .map(|parameter| self.intern(&parameter.ty, line, column))
                    .collect::<Result<Vec<_>, _>>()?,
                result: UNIT,
            },
            _ => return lower(ty, line, column),
        };
        if let Some(definition) = self
            .definitions
            .iter()
            .find(|definition| definition.kind == kind)
        {
            return Ok(definition.id);
        }
        let id = TypeId::try_from_index(self.definitions.len()).map_err(|error| {
            internal_compiler_error(
                format!("Register IR type table limit exceeded: {error}"),
                "Split the program into smaller units.",
                line,
                column,
            )
        })?;
        self.definitions.push(TypeDefinition { id, kind });
        Ok(id)
    }

    pub fn id(&self, ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
        let mut table = self.clone();
        table.intern(ty, line, column)
    }

    pub fn definitions(&self) -> Vec<TypeDefinition> {
        self.definitions.clone()
    }

    pub fn function_result(&self, ty: TypeId) -> Option<TypeId> {
        match self
            .definitions
            .iter()
            .find(|definition| definition.id == ty)
            .map(|definition| &definition.kind)
        {
            Some(IrType::Function { result, .. }) => Some(*result),
            _ => None,
        }
    }

    pub fn type_expr(&mut self, type_expr: &fpas_parser::TypeExpr) -> Result<TypeId, CompileError> {
        use fpas_parser::TypeExpr;
        match type_expr {
            TypeExpr::Named { id, span } => {
                match id.parts.join(".").to_ascii_lowercase().as_str() {
                    "integer" => Ok(INTEGER),
                    "real" => Ok(REAL),
                    "boolean" => Ok(BOOLEAN),
                    "string" => Ok(STRING),
                    _ => Err(type_error("named callable type", *span)),
                }
            }
            TypeExpr::FunctionType {
                params,
                return_type,
                span,
            } => {
                let parameters = params
                    .iter()
                    .map(|parameter| self.type_expr(&parameter.type_expr))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.type_expr(return_type)?;
                self.intern_kind(IrType::Function { parameters, result }, *span)
            }
            TypeExpr::ProcedureType { params, span } => {
                let parameters = params
                    .iter()
                    .map(|parameter| self.type_expr(&parameter.type_expr))
                    .collect::<Result<Vec<_>, _>>()?;
                self.intern_kind(
                    IrType::Function {
                        parameters,
                        result: UNIT,
                    },
                    *span,
                )
            }
            other => Err(type_error("aggregate callable type", type_expr_span(other))),
        }
    }

    pub fn function_type(
        &mut self,
        parameters: Vec<TypeId>,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Function { parameters, result }, span)
    }

    pub fn cell_type(
        &mut self,
        inner: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Cell(inner), span)
    }

    fn intern_kind(
        &mut self,
        kind: IrType,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        if let Some(definition) = self
            .definitions
            .iter()
            .find(|definition| definition.kind == kind)
        {
            return Ok(definition.id);
        }
        let id = TypeId::try_from_index(self.definitions.len())
            .map_err(|error| type_error(&error.to_string(), span))?;
        self.definitions.push(TypeDefinition { id, kind });
        Ok(id)
    }
}

fn type_error(construct: &str, span: fpas_lexer::Span) -> CompileError {
    internal_compiler_error(
        format!("Type `{construct}` is outside the P4 register subset."),
        "Use scalar values and scalar function or procedure types in this development path.",
        span.line,
        span.column,
    )
}

fn type_expr_span(type_expr: &fpas_parser::TypeExpr) -> fpas_lexer::Span {
    use fpas_parser::TypeExpr;
    match type_expr {
        TypeExpr::Named { span, .. }
        | TypeExpr::Array(_, span)
        | TypeExpr::FunctionType { span, .. }
        | TypeExpr::ProcedureType { span, .. }
        | TypeExpr::Result { span, .. }
        | TypeExpr::Option { span, .. }
        | TypeExpr::Dict { span, .. } => *span,
    }
}

pub(super) fn scalar_type_table() -> Vec<TypeDefinition> {
    vec![
        definition(UNIT, IrType::Unit),
        definition(BOOLEAN, IrType::Boolean),
        definition(INTEGER, IrType::Integer),
        definition(REAL, IrType::Real),
        definition(STRING, IrType::String),
        definition(DYNAMIC, IrType::Dynamic),
    ]
}

pub(super) fn lower(ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
    match ty {
        Ty::Unit => Ok(UNIT),
        Ty::Boolean => Ok(BOOLEAN),
        Ty::Integer => Ok(INTEGER),
        Ty::Real => Ok(REAL),
        Ty::String => Ok(STRING),
        Ty::GenericParam(..) => Ok(DYNAMIC),
        other => Err(internal_compiler_error(
            format!("Type `{other}` is outside the P3 scalar register subset."),
            "Use only integer, real, boolean, string, and Unit values in this development path.",
            line,
            column,
        )),
    }
}

fn definition(id: TypeId, kind: IrType) -> TypeDefinition {
    TypeDefinition { id, kind }
}
