//! Borrowed register-window execution for non-hosted standard-library intrinsics.

use fpas_bytecode::{
    AbcOperands, Intrinsic, IntrinsicOwner, NO_REGISTER, Register, SourceLocation, Value,
};

use super::scalar::register;
use crate::vm::hosted::callbacks::CallbackOutcome;
use crate::vm::worker::Worker;
use crate::vm::{VmError, diagnostics};

impl Worker {
    pub(in crate::vm) fn execute_intrinsic(
        &mut self,
        operands: AbcOperands,
    ) -> Result<(), VmError> {
        let intrinsic = Intrinsic::from_u16(operands.b).ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Verified intrinsic identifier {} is unknown", operands.b),
            )
        })?;
        let owner = intrinsic.owner();
        let start = self
            .base
            .checked_add(usize::from(operands.c))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        let end = start
            .checked_add(usize::from(operands.auxiliary))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        if self.registers.get(start..end).is_none() {
            return Err(self.intrinsic_window_error(operands));
        }
        let location = self.intrinsic_location();
        let destination = (operands.a != NO_REGISTER)
            .then(|| register(operands.a))
            .transpose()?;
        if intrinsic.requires_mutable_dispatch() {
            let arguments = self.registers[start..end].to_vec();
            if let Some(result) = self.task_intrinsic(intrinsic, &arguments, destination)? {
                if let (Some(value), Some(destination)) = (result, destination) {
                    self.write(destination, value)?;
                }
                return Ok(());
            }
            let result = self.execute_borrowed_intrinsic(intrinsic, owner, &arguments, location)?;
            return self.store_intrinsic_result(intrinsic, destination, result);
        }
        if self.task_id != 0 && owner == IntrinsicOwner::Callback {
            let arguments = self.registers[start..end].to_vec();
            let absolute_destination =
                destination.map(|register| self.base + usize::from(register.get()));
            if let Some(outcome) = self.execute_callback_intrinsic(
                intrinsic,
                &arguments,
                location,
                absolute_destination,
            )? {
                return match outcome {
                    CallbackOutcome::Complete(value) => {
                        self.store_intrinsic_result(intrinsic, destination, Some(value))
                    }
                    CallbackOutcome::Deferred => Ok(()),
                };
            }
            let result = self.execute_borrowed_intrinsic(intrinsic, owner, &arguments, location)?;
            return self.store_intrinsic_result(intrinsic, destination, result);
        }
        let result = self.execute_borrowed_intrinsic(
            intrinsic,
            owner,
            &self.registers[start..end],
            location,
        )?;
        self.store_intrinsic_result(intrinsic, destination, result)
    }

    pub(in crate::vm) fn execute_debug_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
    ) -> Result<Value, VmError> {
        let base = 1_usize;
        let required = base.saturating_add(arguments.len());
        self.base = 0;
        self.reset_registers(required.max(1));
        for (offset, value) in arguments.iter().enumerate() {
            self.store_register(base + offset, value.clone())?;
        }
        self.execute_intrinsic(AbcOperands {
            a: 0,
            b: intrinsic.into(),
            c: u16::try_from(base).map_err(|_| {
                self.intrinsic_window_error(AbcOperands {
                    a: 0,
                    b: intrinsic.into(),
                    c: u16::MAX,
                    auxiliary: 0,
                })
            })?,
            auxiliary: u8::try_from(arguments.len()).map_err(|_| {
                self.intrinsic_window_error(AbcOperands {
                    a: 0,
                    b: intrinsic.into(),
                    c: 1,
                    auxiliary: u8::MAX,
                })
            })?,
        })?;
        Ok(self.registers[0].clone())
    }

    fn execute_borrowed_intrinsic(
        &self,
        intrinsic: Intrinsic,
        owner: IntrinsicOwner,
        arguments: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Value>, VmError> {
        match owner {
            IntrinsicOwner::Standard => fpas_std::run_intrinsic_borrowed(
                intrinsic,
                arguments,
                location,
                self.layouts.as_ref(),
            ),
            IntrinsicOwner::Hosted => self.execute_hosted_intrinsic(intrinsic, arguments, location),
            IntrinsicOwner::Callback => self
                .execute_callback_intrinsic_sync(intrinsic, arguments, location)?
                .ok_or_else(|| {
                    diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        format!(
                            "Callback intrinsic {intrinsic:?} was not handled by its owning module"
                        ),
                    )
                }),
            IntrinsicOwner::Task => Err(diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Task intrinsic {intrinsic:?} bypassed mutable dispatch"),
            )),
        }
    }

    fn store_intrinsic_result(
        &mut self,
        intrinsic: Intrinsic,
        destination: Option<Register>,
        result: Option<Value>,
    ) -> Result<(), VmError> {
        let Some(destination) = destination else {
            return Ok(());
        };
        let value = result.ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Intrinsic {intrinsic:?} did not produce its verified result"),
            )
        })?;
        self.write(destination, value)
    }

    fn intrinsic_location(&self) -> SourceLocation {
        self.executable
            .executable()
            .source_map
            .lookup(self.current_address)
            .map_or_else(
                || SourceLocation::new(1, 1),
                |run| SourceLocation::new_with_source(run.line, run.column, run.source.get()),
            )
    }

    fn intrinsic_window_error(&self, operands: AbcOperands) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!(
                "Intrinsic register window {}..+{} is outside the current frame",
                operands.c, operands.auxiliary
            ),
        )
    }
}
