//! Runtime-shared positional aggregate names derived from verified metadata.

use std::sync::Arc;

use fpas_bytecode::{
    EnumVariantId, Executable, InstructionAddress, PositionalEnumLayout, PositionalRecordLayout,
    RecordTypeId,
};

use super::{VmError, diagnostics};

pub(super) struct RuntimeLayouts {
    pub records: Vec<Arc<PositionalRecordLayout>>,
    pub enum_variants: Vec<Arc<PositionalEnumLayout>>,
}

impl RuntimeLayouts {
    pub fn build(executable: &Executable, address: InstructionAddress) -> Result<Self, VmError> {
        let records = executable
            .records
            .iter()
            .enumerate()
            .map(|(index, layout)| {
                let record = RecordTypeId::try_from_index(index).map_err(|error| {
                    diagnostics::internal(executable, address, error.to_string())
                })?;
                let type_name = string(executable, layout.name, address)?.to_owned();
                let fields = layout
                    .fields
                    .iter()
                    .map(|field| string(executable, field.name, address).map(str::to_owned))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Arc::new(PositionalRecordLayout {
                    record,
                    type_name,
                    fields,
                }))
            })
            .collect::<Result<Vec<_>, VmError>>()?;
        let enum_variants = executable
            .enum_variants
            .iter()
            .enumerate()
            .map(|(index, variant)| {
                let variant_id = EnumVariantId::try_from_index(index).map_err(|error| {
                    diagnostics::internal(executable, address, error.to_string())
                })?;
                let owner = executable
                    .enums
                    .get(usize::from(variant.owner.get()))
                    .ok_or_else(|| {
                        diagnostics::internal(
                            executable,
                            address,
                            "Verified enum owner is unavailable",
                        )
                    })?;
                let type_name = string(executable, owner.name, address)?.to_owned();
                let variant_name = string(executable, variant.name, address)?.to_owned();
                let fields = variant
                    .fields
                    .iter()
                    .map(|field| string(executable, *field, address).map(str::to_owned))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Arc::new(PositionalEnumLayout {
                    enumeration: variant.owner,
                    variant_id,
                    type_name,
                    variant: variant_name,
                    fields,
                }))
            })
            .collect::<Result<Vec<_>, VmError>>()?;
        Ok(Self {
            records,
            enum_variants,
        })
    }
}

fn string(
    executable: &Executable,
    id: fpas_bytecode::StringId,
    address: InstructionAddress,
) -> Result<&str, VmError> {
    executable.strings.get(id).ok_or_else(|| {
        diagnostics::internal(
            executable,
            address,
            "Verified aggregate layout string is unavailable",
        )
    })
}
