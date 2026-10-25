//! Runtime-shared positional aggregate names derived from verified metadata.

mod native_errors;
mod native_records;
use std::sync::Arc;

use fpas_bytecode::{
    EnumVariantId, Executable, InstructionAddress, RecordTypeId, RuntimeEnumLayout,
    RuntimeRecordLayout, SharedEnum, SharedRecord, SourceLocation, Value,
};

use super::{VmError, diagnostics};

pub(super) struct RuntimeLayouts {
    native_errors: Vec<Vec<Option<Arc<RuntimeEnumLayout>>>>,
    pub records: Vec<Arc<RuntimeRecordLayout>>,
    pub enum_variants: Vec<Arc<RuntimeEnumLayout>>,
}

impl RuntimeLayouts {
    pub fn build(executable: &Executable, address: InstructionAddress) -> Result<Self, VmError> {
        native_errors::validate(executable, address)?;
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
                    error: variant
                        .error
                        .as_ref()
                        .map(|error| {
                            let mut runtime = fpas_bytecode::RuntimeErrorVariant::from_metadata(
                                error,
                                &executable.strings,
                            )?;
                            runtime.hide_payload = fpas_std::native_errors::NativeErrorFamily::ALL
                                .iter()
                                .any(|family| {
                                    family.name().eq_ignore_ascii_case(&type_name)
                                        && family.hide_payload()
                                });
                            Ok::<_, fpas_bytecode::EnumConstructionError>(runtime)
                        })
                        .transpose()
                        .map_err(|error| {
                            diagnostics::internal(executable, address, error.to_string())
                        })?,
                    enumeration: variant.owner,
                    variant_id,
                    type_name,
                    variant: variant_name,
                    fields,
                }))
            })
            .collect::<Result<Vec<_>, VmError>>()?;
        let native_errors = fpas_std::native_errors::NativeErrorFamily::ALL
            .iter()
            .map(|family| {
                family
                    .variants()
                    .iter()
                    .map(|variant| {
                        enum_variants
                            .iter()
                            .find(|layout| {
                                layout.type_name.eq_ignore_ascii_case(family.name())
                                    && layout.variant.eq_ignore_ascii_case(variant.name)
                            })
                            .cloned()
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            records,
            enum_variants,
            native_errors,
        })
    }

    /// Copy record and enum layouts into allocations owned by one pool thread.
    ///
    /// Every aggregate value retains its layout, so one allocation shared by all threads
    /// makes their reference counts contend. Layouts compare by content, not identity.
    /// Native error layouts stay shared because those values are rare.
    pub fn detached(&self) -> Self {
        fn copy<T: Clone>(layouts: &[Arc<T>]) -> Vec<Arc<T>> {
            layouts
                .iter()
                .map(|layout| Arc::new(T::clone(layout)))
                .collect()
        }
        Self {
            records: copy(&self.records),
            enum_variants: copy(&self.enum_variants),
            native_errors: self.native_errors.clone(),
        }
    }
}

impl fpas_std::AggregateFactory for RuntimeLayouts {
    fn native_error(
        &self,
        family: fpas_std::native_errors::NativeErrorFamily,
        variant: usize,
        values: Vec<Value>,
        location: SourceLocation,
    ) -> Result<Value, fpas_std::StdError> {
        let layout = self
            .native_errors
            .get(family as usize)
            .and_then(|variants| variants.get(variant))
            .and_then(Clone::clone)
            .ok_or_else(|| missing_layout("native error", family.name(), location))?;
        SharedEnum::try_new(layout, values)
            .map(Value::Enum)
            .map_err(|error| {
                fpas_diagnostics::Diagnostic::error(
                    fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
                    error.to_string(),
                    None,
                    fpas_diagnostics::SourceSpan::synthetic_from_location(location),
                )
                .into()
            })
    }

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
        SharedEnum::try_new(layout, values)
            .map(Value::Enum)
            .map_err(|error| {
                fpas_diagnostics::Diagnostic::error(
                    fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
                    error.to_string(),
                    None,
                    fpas_diagnostics::SourceSpan::synthetic_from_location(location),
                )
                .into()
            })
    }
}

fn missing_layout(kind: &str, name: &str, location: SourceLocation) -> fpas_std::StdError {
    fpas_diagnostics::Diagnostic::error(
        fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE,
        format!("Verified {kind} layout `{name}` is unavailable"),
        Some("Recompile the program and report this compiler/runtime layout mismatch.".into()),
        fpas_diagnostics::SourceSpan::synthetic_from_location(location),
    )
    .into()
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
        fpas_diagnostics::SourceSpan::synthetic_from_location(location),
    )
    .into()
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

#[cfg(test)]
mod native_errors_tests;
