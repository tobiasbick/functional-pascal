//! Compact semantic-to-IR scalar type mapping.

mod expressions;
mod layouts;

use fpas_ir::{
    EnumLayout, EnumLayoutId, FieldId, IrType, RecordLayout, RecordLayoutId, TypeDefinition,
    TypeId, VariantId,
};
use fpas_sema::Ty;
use std::collections::{BTreeMap, BTreeSet};

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
    filled_record_layouts: BTreeSet<RecordLayoutId>,
    filled_enum_layouts: BTreeSet<EnumLayoutId>,
    simple_enums: BTreeSet<String>,
    named: BTreeMap<String, TypeId>,
}

impl TypeTable {
    pub fn from_metadata(metadata: &fpas_sema::AnalysisMetadata) -> Result<Self, CompileError> {
        let mut table = Self {
            definitions: scalar_type_table(),
            record_layouts: Vec::new(),
            enum_layouts: Vec::new(),
            filled_record_layouts: BTreeSet::new(),
            filled_enum_layouts: BTreeSet::new(),
            simple_enums: BTreeSet::new(),
            named: BTreeMap::new(),
        };
        for (name, ty) in &metadata.named_types {
            let id = match ty {
                Ty::Record(record) => {
                    let layout = table.reserve_record(record, 1, 1)?;
                    table.intern_kind(IrType::Record(layout), synthetic_span(1, 1))?
                }
                Ty::Enum(enumeration) if enumeration.has_data() => {
                    let layout = table.reserve_enum(enumeration, 1, 1)?;
                    table.intern_kind(IrType::Enum(layout), synthetic_span(1, 1))?
                }
                Ty::Enum(enumeration) => {
                    table
                        .simple_enums
                        .insert(enumeration.name.to_ascii_lowercase());
                    INTEGER
                }
                _ => continue,
            };
            table.named.insert(name.to_ascii_lowercase(), id);
        }
        for (name, ty) in &metadata.named_types {
            if matches!(ty, Ty::Error | Ty::Named(_)) {
                continue;
            }
            let id = table.intern(ty, 1, 1)?;
            table.named.insert(name.to_ascii_lowercase(), id);
        }
        for ty in metadata.expr_types.values() {
            if matches!(ty, Ty::Error | Ty::Named(_)) {
                continue;
            }
            let _ = table.intern(ty, 1, 1)?;
        }
        let dictionary_keys = table
            .definitions
            .iter()
            .filter_map(|definition| match definition.kind {
                IrType::Dictionary { key, .. } => Some(key),
                _ => None,
            })
            .collect::<Vec<_>>();
        for key in dictionary_keys {
            let _ = table.intern_kind(IrType::Array(key), synthetic_span(1, 1))?;
        }
        if metadata
            .expr_types
            .values()
            .any(|ty| matches!(ty, Ty::Task(_)))
        {
            let outputs = table
                .definitions
                .iter()
                .filter(|definition| !matches!(definition.kind, IrType::Task(_)))
                .map(|definition| definition.id)
                .collect::<Vec<_>>();
            for output in outputs {
                let _ = table.intern_kind(IrType::Task(output), synthetic_span(1, 1))?;
            }
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
            Ty::Named(name) => {
                return Ok(self.named_type(name).unwrap_or(DYNAMIC));
            }
            Ty::Error => return Ok(DYNAMIC),
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

    fn named_type(&self, name: &str) -> Option<TypeId> {
        if let Some(id) = self.named.get(&name.to_ascii_lowercase()) {
            return Some(*id);
        }
        let mut candidates = self
            .named
            .iter()
            .filter(|(candidate, _)| super::type_names::matches(candidate, name))
            .map(|(_, id)| *id)
            .collect::<BTreeSet<_>>();
        for layout in &self.record_layouts {
            if super::type_names::matches(&layout.name, name)
                && let Some(id) = self.definitions.iter().find_map(|definition| {
                    (definition.kind == IrType::Record(layout.id)).then_some(definition.id)
                })
            {
                candidates.insert(id);
            }
        }
        for layout in &self.enum_layouts {
            if super::type_names::matches(&layout.name, name)
                && let Some(id) = self.definitions.iter().find_map(|definition| {
                    (definition.kind == IrType::Enum(layout.id)).then_some(definition.id)
                })
            {
                candidates.insert(id);
            }
        }
        if self
            .simple_enums
            .iter()
            .any(|enumeration| super::type_names::matches(enumeration, name))
        {
            candidates.insert(INTEGER);
        }
        let mut candidates = candidates.into_iter();
        let id = candidates.next()?;
        candidates.next().is_none().then_some(id)
    }

    pub fn task_type(&self, inner: TypeId) -> Option<TypeId> {
        self.definitions
            .iter()
            .find(|definition| definition.kind == IrType::Task(inner))
            .map(|definition| definition.id)
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

    pub fn record_fields(&self, layout: RecordLayoutId) -> Option<Vec<(String, TypeId)>> {
        self.record_layouts
            .iter()
            .find(|item| item.id == layout)
            .map(|item| {
                item.fields
                    .iter()
                    .map(|field| (field.name.clone(), field.ty))
                    .collect()
            })
    }

    pub fn record_layout_name(&self, layout: RecordLayoutId) -> Option<&str> {
        self.record_layouts
            .iter()
            .find(|item| item.id == layout)
            .map(|item| item.name.as_str())
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

    pub fn function_type(
        &mut self,
        parameters: Vec<TypeId>,
        result: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Function { parameters, result }, span)
    }

    pub fn array_type(
        &mut self,
        element: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Array(element), span)
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
        format!("The compiler could not lower type `{construct}`."),
        "This is an internal compiler error. Re-run compilation and report the source program.",
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
            format!("The compiler could not lower type `{other}`."),
            "This is an internal compiler error. Re-run compilation and report the source program.",
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
