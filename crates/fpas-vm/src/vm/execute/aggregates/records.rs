use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, internal_error, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    pub(super) fn exec_make_record(
        &mut self,
        type_idx: u16,
        field_count: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let type_name = self.const_str(type_idx, line)?;
        let items = self.drain_stack_tail(field_count as usize * 2, line)?;
        let fields = items
            .chunks(2)
            .map(|pair| {
                let Value::Str(name) = &pair[0] else {
                    return Err(internal_error(
                        "MakeRecord expected string field names",
                        "This indicates invalid bytecode or a compiler record-lowering bug. Please report it.",
                        line,
                    ));
                };
                Ok((name.clone(), pair[1].clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
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

        Err(runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "FieldGet requires a record value",
            "Use field access only on record values.",
            line,
        ))
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
            let Some(entry) = fields.iter_mut().find(|(name, _)| name == &field_name) else {
                return Err(missing_field_error(&field_name, line));
            };
            entry.1 = value;
            self.push(Value::Record { type_name, fields })?;
            return Ok(());
        }

        Err(runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            "FieldSet requires a record value",
            "Use field assignment only on record values.",
            line,
        ))
    }
}

impl Worker {
    /// Execute `UpdateRecord(n)`: copy a record, overriding `n` named fields.
    ///
    /// Stack before: `[base_record, name0, val0, …, nameN-1, valN-1]`  
    /// Stack after: `[new_record]`  
    ///
    /// **Documentation:** `docs/pascal/05-types.md` (Record Update Expression)
    pub(super) fn exec_update_record(
        &mut self,
        n_overrides: u16,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        // Drain N (name, value) pairs pushed AFTER the base record.
        let override_items = self.drain_stack_tail(n_overrides as usize * 2, line)?;
        let base = self.pop(line)?;

        let Value::Record {
            type_name,
            mut fields,
        } = base
        else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                "`with` update requires a record value",
                "Use `RecordExpr with Field := NewValue; … end` on a record value.",
                line,
            ));
        };

        for pair in override_items.chunks(2) {
            let Value::Str(name) = &pair[0] else {
                return Err(internal_error(
                    "UpdateRecord expected string field names",
                    "This indicates invalid bytecode or a compiler record-update lowering bug. Please report it.",
                    line,
                ));
            };
            if let Some(entry) = fields.iter_mut().find(|(n, _)| n == name) {
                entry.1 = pair[1].clone();
            } else {
                return Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Record has no field `{name}` to update"),
                    "Check the field name against the record type definition.",
                    line,
                ));
            }
        }

        self.push(Value::Record { type_name, fields })?;
        Ok(())
    }
}

fn missing_field_error(field_name: &str, line: SourceLocation) -> VmError {
    runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("Record has no field `{field_name}`"),
        "Check the field name against the record type definition.",
        line,
    )
}
