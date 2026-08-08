//! Runtime-shared positional aggregate names derived from verified metadata.

use std::sync::Arc;

use fpas_bytecode::{
    EnumVariantId, Executable, InstructionAddress, RecordTypeId, RuntimeEnumLayout,
    RuntimeRecordLayout, SharedEnum, SharedRecord, SourceLocation, Value,
};

use super::{VmError, diagnostics};

pub(super) struct RuntimeLayouts {
    pub records: Vec<Arc<RuntimeRecordLayout>>,
    pub enum_variants: Vec<Arc<RuntimeEnumLayout>>,
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
                Ok(Arc::new(RuntimeRecordLayout {
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
                Ok(Arc::new(RuntimeEnumLayout {
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

impl fpas_std::AggregateFactory for RuntimeLayouts {
    fn record(
        &self,
        type_name: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, fpas_std::StdError> {
        let layout = self
            .records
            .iter()
            .find(|layout| layout.type_name.eq_ignore_ascii_case(type_name))
            .cloned()
            .ok_or_else(|| missing_layout("record", type_name, location))?;
        if layout.fields.len() != values.len() {
            return Err(layout_arity_error(
                type_name,
                layout.fields.len(),
                values.len(),
                location,
            ));
        }
        Ok(Value::Record(SharedRecord::new(layout, values)))
    }

    fn enumeration(
        &self,
        type_name: &str,
        variant: &str,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, fpas_std::StdError> {
        let layout = self
            .enum_variants
            .iter()
            .find(|layout| {
                layout.type_name.eq_ignore_ascii_case(type_name)
                    && layout.variant.eq_ignore_ascii_case(variant)
            })
            .cloned()
            .ok_or_else(|| {
                missing_layout("enum variant", &format!("{type_name}.{variant}"), location)
            })?;
        if layout.fields.len() != values.len() {
            return Err(layout_arity_error(
                &format!("{type_name}.{variant}"),
                layout.fields.len(),
                values.len(),
                location,
            ));
        }
        Ok(Value::Enum(SharedEnum::new(layout, values)))
    }
}

fn missing_layout(kind: &str, name: &str, location: SourceLocation) -> fpas_std::StdError {
    fpas_diagnostics::Diagnostic::error(
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        format!("Verified {kind} layout `{name}` is unavailable"),
        Some("Recompile the program and report this compiler/runtime layout mismatch.".into()),
        fpas_diagnostics::SourceSpan::new(0, 1, location.line(), location.column()),
    )
}

fn layout_arity_error(
    name: &str,
    expected: usize,
    actual: usize,
    location: SourceLocation,
) -> fpas_std::StdError {
    fpas_diagnostics::Diagnostic::error(
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        format!("Aggregate `{name}` expects {expected} fields, received {actual}"),
        Some("Recompile the program and report this compiler/runtime layout mismatch.".into()),
        fpas_diagnostics::SourceSpan::new(0, 1, location.line(), location.column()),
    )
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
