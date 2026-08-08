//! Register-window selection for standard-library and hosted intrinsic calls.

use fpas_bytecode::{Instruction, NO_REGISTER, Opcode};
use fpas_ir::{IrType, Operation, ValueId};

use crate::CompileError;

use super::{Selector, abc, abc_aux, argument_count, selection_error};

impl Selector<'_> {
    pub(super) fn select_intrinsic(
        &self,
        operation: &Operation,
        result: Option<ValueId>,
    ) -> Result<Option<Vec<Instruction>>, CompileError> {
        let Operation::Intrinsic {
            intrinsic,
            arguments,
        } = operation
        else {
            return Ok(None);
        };
        let wire = u16::try_from(intrinsic.get())
            .map_err(|_| selection_error("intrinsic identifier exceeds u16"))?;
        fpas_bytecode::Intrinsic::from_u16(wire)
            .ok_or_else(|| selection_error("intrinsic identifier has no stable wire variant"))?;
        let mut instructions = self.prepare_window(arguments)?;
        let result_id = result.ok_or_else(|| selection_error("intrinsic call has no result"))?;
        let returns_unit = matches!(
            self.value_types
                .get(&result_id)
                .and_then(|ty| self.program.ty(*ty))
                .map(|definition| &definition.kind),
            Some(IrType::Unit)
        );
        let destination = if returns_unit {
            NO_REGISTER
        } else {
            self.result_register(result)?
        };
        instructions.push(abc_aux(
            Opcode::Intrinsic,
            destination,
            wire,
            self.allocation.call_window().get(),
            argument_count(arguments)?,
        )?);
        if returns_unit {
            instructions.push(abc(Opcode::LoadUnit, self.result_register(result)?, 0, 0)?);
        }
        Ok(Some(instructions))
    }
}
