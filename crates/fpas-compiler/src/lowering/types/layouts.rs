//! Two-phase record and enum layout reservation and population.

use fpas_ir::{
    EnumLayout, EnumLayoutId, EnumVariant, FieldId, RecordField, RecordLayout, RecordLayoutId,
    VariantId,
};

use crate::CompileError;

use super::{TypeTable, synthetic_span, type_error};

impl TypeTable {
    /// Populate the canonical record layout, using erased declaration fields for generics.
    pub(super) fn intern_record(
        &mut self,
        record: &fpas_sema::RecordTy,
        line: u32,
        column: u32,
    ) -> Result<RecordLayoutId, CompileError> {
        let template = self
            .generic_templates
            .get(&record.name.to_ascii_lowercase())
            .and_then(|ty| match ty {
                fpas_sema::Ty::Record(record) => Some(record.clone()),
                _ => None,
            });
        let record = template.as_deref().unwrap_or(record);
        let id = if let Some(layout) = self
            .record_layouts
            .iter()
            .find(|layout| layout.name.eq_ignore_ascii_case(&record.name))
        {
            layout.id
        } else {
            self.reserve_record(record, line, column)?
        };
        if !self.filled_record_layouts.insert(id) {
            return Ok(id);
        }
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

    pub(super) fn reserve_record(
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
            construction: Some(fpas_ir::RecordConstructionInfo {
                owner_unit: record.owner_unit.clone(),
                requires_owner: record.owner_unit.is_some()
                    && record.fields.iter().any(|(name, _)| {
                        record
                            .private_members
                            .iter()
                            .any(|private| private.eq_ignore_ascii_case(name))
                    }),
                aliases: Vec::new(),
                scope_unit: String::new(),
                defaults: vec![None; record.fields.len()],
            }),
            methods: record
                .methods
                .iter()
                .map(|(name, _)| fpas_ir::RecordMethod {
                    name: name.clone(),
                    routine: format!("{}.{}", record.name, name),
                })
                .collect(),
        });
        Ok(id)
    }

    /// Populate the canonical enum layout, including recursive generic payloads.
    pub(super) fn intern_enum(
        &mut self,
        enumeration: &fpas_sema::EnumTy,
        line: u32,
        column: u32,
    ) -> Result<EnumLayoutId, CompileError> {
        let template = self
            .generic_templates
            .get(&enumeration.name.to_ascii_lowercase())
            .and_then(|ty| match ty {
                fpas_sema::Ty::Enum(enumeration) => Some(enumeration.clone()),
                _ => None,
            });
        let enumeration = template.as_deref().unwrap_or(enumeration);
        let id = if let Some(layout) = self
            .enum_layouts
            .iter()
            .find(|layout| layout.name.eq_ignore_ascii_case(&enumeration.name))
        {
            layout.id
        } else {
            self.reserve_enum(enumeration, line, column)?
        };
        if !self.filled_enum_layouts.insert(id) {
            return Ok(id);
        }
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

    pub(super) fn reserve_enum(
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
        Ok(id)
    }
}
