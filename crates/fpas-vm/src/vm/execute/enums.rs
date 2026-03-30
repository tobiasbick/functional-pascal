//! VM execution for enum variant opcodes (`MakeEnum`, `IsVariant`, `EnumField`).
//!
//! **Documentation:** `docs/future/advanced-types.md`

use super::super::diagnostics::VmError;
use super::super::{Worker, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

impl Worker {
    pub(super) fn try_exec_enums(&mut self, op: Op, line: SourceLocation) -> Result<bool, VmError> {
        match op {
            Op::MakeEnum(type_idx, variant_idx, field_count) => {
                let type_name = self.const_str(type_idx, line)?;
                let variant = self.const_str(variant_idx, line)?;
                let fields = self.drain_stack_tail(field_count as usize, line)?;
                self.push(Value::Enum {
                    type_name,
                    variant,
                    fields,
                })?;
                Ok(true)
            }
            Op::IsVariant(name_idx) => {
                let expected = self.const_str(name_idx, line)?;
                let val = self.pop(line)?;
                let matches = match &val {
                    Value::Enum { variant, .. } => *variant == expected,
                    _ => false,
                };
                self.push(Value::Boolean(matches))?;
                Ok(true)
            }
            Op::EnumField(index) => {
                let val = self.pop(line)?;
                match val {
                    Value::Enum {
                        fields, variant, ..
                    } => {
                        let idx = index as usize;
                        if idx >= fields.len() {
                            return Err(runtime_error(
                                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                                format!(
                                    "Enum variant `{variant}` has {} field(s), tried to access index {idx}",
                                    fields.len()
                                ),
                                "Check the number of fields in the variant definition.",
                                line,
                            ));
                        }
                        self.push(fields[idx].clone())?;
                        Ok(true)
                    }
                    other => Err(runtime_error(
                        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                        format!("EnumField expected enum value, got {}", other.type_name()),
                        "Use EnumField only on enum values with associated data.",
                        line,
                    )),
                }
            }
            _ => Ok(false),
        }
    }
}
