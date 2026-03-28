use super::super::super::{Vm, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Vm {
    pub(super) fn exec_make_record(
        &mut self,
        type_idx: u16,
        field_count: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let type_name = self.const_str(type_idx, line)?;
        let items = self.drain_values(field_count as usize * 2);
        let fields = items
            .chunks(2)
            .map(|pair| {
                let name = match &pair[0] {
                    Value::Str(name) => name.clone(),
                    _ => String::new(),
                };
                (name, pair[1].clone())
            })
            .collect();
        self.push(Value::Record { type_name, fields })?;
        Ok(())
    }

    pub(super) fn exec_field_get(
        &mut self,
        name_idx: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let field_name = self.const_str(name_idx, line)?;
        let record = self.pop(line)?;
        if let Value::Record { fields, .. } = record {
            let value = fields
                .iter()
                .find(|(name, _)| name == &field_name)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| {
                    runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("Record has no field `{field_name}`"),
                        "Check the field name against the record type definition.",
                        line,
                    )
                })?;
            self.push(value)?;
            return Ok(());
        }

        Err(record_operand_error("FieldGet", line))
    }

    pub(super) fn exec_field_set(
        &mut self,
        name_idx: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let field_name = self.const_str(name_idx, line)?;
        let value = self.pop(line)?;
        let record = self.pop(line)?;
        if let Value::Record {
            type_name,
            mut fields,
        } = record
        {
            if let Some(entry) = fields.iter_mut().find(|(name, _)| name == &field_name) {
                entry.1 = value;
            }
            self.push(Value::Record { type_name, fields })?;
            return Ok(());
        }

        Err(record_operand_error("FieldSet", line))
    }
}

fn record_operand_error(op_name: &str, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("{op_name} requires a record"),
        "Use field access only on record values.",
        line,
    )
}
