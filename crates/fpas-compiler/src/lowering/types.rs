//! Compact semantic-to-IR scalar type mapping.

use fpas_ir::{
    EnumLayout, EnumLayoutId, EnumVariant, FieldId, IrType, RecordField, RecordLayout,
    RecordLayoutId, TypeDefinition, TypeId, VariantId,
};
use fpas_sema::Ty;
use std::collections::BTreeSet;

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
    record_layouts: Vec<RecordLayout>,
    enum_layouts: Vec<EnumLayout>,
    simple_enums: BTreeSet<String>,
}

impl TypeTable {
    pub fn from_metadata(metadata: &fpas_sema::AnalysisMetadata) -> Result<Self, CompileError> {
        let mut table = Self {
            definitions: scalar_type_table(),
            record_layouts: Vec::new(),
            enum_layouts: Vec::new(),
            simple_enums: BTreeSet::new(),
        };
        for ty in metadata.named_types.values() {
            if matches!(ty, Ty::Error | Ty::Named(_)) {
                continue;
            }
            let _ = table.intern(ty, 1, 1)?;
        }
        for ty in metadata.expr_types.values() {
            if matches!(ty, Ty::Error | Ty::Named(_)) {
                continue;
            }
            let _ = table.intern(ty, 1, 1)?;
        }
        Ok(table)
    }

    pub fn intern(&mut self, ty: &Ty, line: u32, column: u32) -> Result<TypeId, CompileError> {
        let kind = match ty {
            Ty::Array(element) => IrType::Array(self.intern(element, line, column)?),
            Ty::Dict(key, value) => IrType::Dictionary {
                key: self.intern(key, line, column)?,
                value: self.intern(value, line, column)?,
            },
            Ty::Result(ok, error) => IrType::Result {
                ok: self.intern(ok, line, column)?,
                error: self.intern(error, line, column)?,
            },
            Ty::Option(value) => IrType::Option(self.intern(value, line, column)?),
            Ty::Task(value) => IrType::Task(self.intern(value, line, column)?),
            Ty::Record(record) => {
                let layout = self.intern_record(record, line, column)?;
                IrType::Record(layout)
            }
            Ty::Enum(enumeration) if enumeration.has_data() => {
                let layout = self.intern_enum(enumeration, line, column)?;
                IrType::Enum(layout)
            }
            Ty::Enum(enumeration) => {
                self.simple_enums
                    .insert(enumeration.name.to_ascii_lowercase());
                return Ok(INTEGER);
            }
            Ty::Error | Ty::Named(_) => return Ok(DYNAMIC),
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

    pub fn record_layouts(&self) -> Vec<RecordLayout> {
        self.record_layouts.clone()
    }

    pub fn enum_layouts(&self) -> Vec<EnumLayout> {
        self.enum_layouts.clone()
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

    pub fn kind(&self, ty: TypeId) -> Option<&IrType> {
        self.definitions
            .iter()
            .find(|definition| definition.id == ty)
            .map(|definition| &definition.kind)
    }

    pub fn record_field(&self, layout: RecordLayoutId, name: &str) -> Option<(FieldId, TypeId)> {
        self.record_layouts
            .iter()
            .find(|item| item.id == layout)?
            .fields
            .iter()
            .find(|field| field.name.eq_ignore_ascii_case(name))
            .map(|field| (field.id, field.ty))
    }

    pub fn record_layout_id(&self, ty: TypeId) -> Option<RecordLayoutId> {
        match self.kind(ty) {
            Some(IrType::Record(layout)) => Some(*layout),
            _ => None,
        }
    }

    pub fn enum_variant(
        &self,
        layout: EnumLayoutId,
        name: &str,
    ) -> Option<(VariantId, Vec<TypeId>)> {
        self.enum_layouts
            .iter()
            .find(|item| item.id == layout)?
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(name))
            .map(|variant| (variant.id, variant.fields.clone()))
    }

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
                match name.to_ascii_lowercase().as_str() {
                    "integer" => Ok(INTEGER),
                    "real" => Ok(REAL),
                    "boolean" => Ok(BOOLEAN),
                    "string" => Ok(STRING),
                    _ => {
                        if let Some(layout) = self
                            .record_layouts
                            .iter()
                            .find(|layout| layout.name.eq_ignore_ascii_case(&name))
                        {
                            return self.intern_kind(IrType::Record(layout.id), *span);
                        }
                        if let Some(layout) = self
                            .enum_layouts
                            .iter()
                            .find(|layout| layout.name.eq_ignore_ascii_case(&name))
                        {
                            return self.intern_kind(IrType::Enum(layout.id), *span);
                        }
                        if self.simple_enums.contains(&name.to_ascii_lowercase()) {
                            return Ok(INTEGER);
                        }
                        Err(type_error("named callable type", *span))
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

    fn intern_record(
        &mut self,
        record: &fpas_sema::RecordTy,
        line: u32,
        column: u32,
    ) -> Result<RecordLayoutId, CompileError> {
        if let Some(layout) = self
            .record_layouts
            .iter()
            .find(|layout| layout.name.eq_ignore_ascii_case(&record.name))
        {
            return Ok(layout.id);
        }
        let id = RecordLayoutId::try_from_index(self.record_layouts.len())
            .map_err(|error| type_error(&error.to_string(), synthetic_span(line, column)))?;
        self.record_layouts.push(RecordLayout {
            id,
            name: record.name.clone(),
            fields: Vec::new(),
        });
        let fields = record
            .fields
            .iter()
            .enumerate()
            .map(|(index, (name, ty))| {
                Ok(RecordField {
                    id: FieldId::try_from_index(index).map_err(|error| {
                        type_error(&error.to_string(), synthetic_span(line, column))
                    })?,
                    name: name.clone(),
                    ty: self.intern(ty, line, column)?,
                })
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        self.record_layouts[usize::try_from(id.get())
            .map_err(|_| type_error("record layout", synthetic_span(line, column)))?]
        .fields = fields;
        Ok(id)
    }

    fn intern_enum(
        &mut self,
        enumeration: &fpas_sema::EnumTy,
        line: u32,
        column: u32,
    ) -> Result<EnumLayoutId, CompileError> {
        if let Some(layout) = self
            .enum_layouts
            .iter()
            .find(|layout| layout.name.eq_ignore_ascii_case(&enumeration.name))
        {
            return Ok(layout.id);
        }
        let id = EnumLayoutId::try_from_index(self.enum_layouts.len())
            .map_err(|error| type_error(&error.to_string(), synthetic_span(line, column)))?;
        self.enum_layouts.push(EnumLayout {
            id,
            name: enumeration.name.clone(),
            variants: Vec::new(),
        });
        let variants = enumeration
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let (field_names, fields): (Vec<_>, Vec<_>) = variant
                    .fields
                    .iter()
                    .map(|(name, ty)| Ok((name.clone(), self.intern(ty, line, column)?)))
                    .collect::<Result<Vec<_>, CompileError>>()?
                    .into_iter()
                    .unzip();
                Ok(EnumVariant {
                    id: VariantId::try_from_index(index).map_err(|error| {
                        type_error(&error.to_string(), synthetic_span(line, column))
                    })?,
                    name: variant.name.clone(),
                    field_names,
                    fields,
                })
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        self.enum_layouts[usize::try_from(id.get())
            .map_err(|_| type_error("enum layout", synthetic_span(line, column)))?]
        .variants = variants;
        Ok(id)
    }
}

fn type_error(construct: &str, span: fpas_lexer::Span) -> CompileError {
    internal_compiler_error(
        format!("Type `{construct}` is outside the P5 register subset."),
        "Use a supported scalar, aggregate, function, or procedure type in this development path.",
        span.line,
        span.column,
    )
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
        Ty::Enum(enumeration) if !enumeration.has_data() => Ok(INTEGER),
        Ty::Error | Ty::Named(_) => Ok(DYNAMIC),
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

fn synthetic_span(line: u32, column: u32) -> fpas_lexer::Span {
    fpas_lexer::Span {
        offset: 0,
        length: 0,
        line,
        column,
        source_id: 0,
    }
}
