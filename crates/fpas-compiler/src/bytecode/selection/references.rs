//! Checked fixed-width selection for synchronous caller references.

use fpas_bytecode::{Instruction, Opcode};
use fpas_ir::{ReferenceOperation, ValueId};

use super::{Selector, abc, narrow};
use crate::CompileError;

impl Selector<'_> {
    /// Select reservation, projection, snapshot, write and release instructions.
    pub(super) fn select_reference(
        &self,
        operation: &ReferenceOperation,
        result: Option<ValueId>,
    ) -> Result<Vec<Instruction>, CompileError> {
        let selected = match operation {
            ReferenceOperation::Reserve(root) => abc(
                Opcode::ReserveReference,
                self.result_register(result)?,
                self.allocation.value(*root)?.get(),
                0,
            ),
            ReferenceOperation::Field {
                reference,
                layout,
                field,
            } => {
                let destination = self.result_register(result)?;
                let source = self.allocation.value(*reference)?.get();
                let mut instructions = Vec::new();
                if source != destination {
                    instructions.push(abc(Opcode::Move, destination, source, 0)?);
                }
                instructions.push(abc(
                    Opcode::SelectReferenceField,
                    destination,
                    narrow(layout.get(), "reference record layout")?,
                    narrow(field.get(), "reference record field")?,
                )?);
                return Ok(instructions);
            }
            ReferenceOperation::Index { reference, index } => abc(
                Opcode::SelectReferenceIndex,
                self.result_register(result)?,
                self.allocation.value(*reference)?.get(),
                self.allocation.value(*index)?.get(),
            ),
            ReferenceOperation::Read(reference) => abc(
                Opcode::ReadReference,
                self.result_register(result)?,
                self.allocation.value(*reference)?.get(),
                0,
            ),
            ReferenceOperation::Write { reference, value } => abc(
                Opcode::WriteReference,
                self.allocation.value(*reference)?.get(),
                self.allocation.value(*value)?.get(),
                0,
            ),
            ReferenceOperation::Release(reference) => abc(
                Opcode::ReleaseReference,
                self.allocation.value(*reference)?.get(),
                0,
                0,
            ),
        }?;
        Ok(vec![selected])
    }
}
