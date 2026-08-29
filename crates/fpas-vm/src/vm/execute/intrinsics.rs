//! Borrowed register-window execution for non-hosted standard-library intrinsics.

use fpas_bytecode::{AbcOperands, Intrinsic, NO_REGISTER, SourceLocation, Value};

use super::scalar::register;
use crate::vm::hosted::HostedOutcome;
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
        let start = self
            .base
            .checked_add(usize::from(operands.c))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        let end = start
            .checked_add(usize::from(operands.auxiliary))
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        let arguments = self
            .registers
            .get(start..end)
            .ok_or_else(|| self.intrinsic_window_error(operands))?
            .to_vec();
        let location = self.intrinsic_location();
        let destination = (operands.a != NO_REGISTER)
            .then(|| register(operands.a))
            .transpose()?;
        let outcome = if matches!(intrinsic, Intrinsic::Task(_) | Intrinsic::Time(_)) {
            let destination = (operands.a != NO_REGISTER)
                .then(|| register(operands.a))
                .transpose()?;
            if let Some(result) = self.task_intrinsic(intrinsic, &arguments, destination)? {
                if let (Some(value), false) = (result, operands.a == NO_REGISTER) {
                    self.write(register(operands.a)?, value)?;
                }
                return Ok(());
            }
            self.execute_borrowed_intrinsic(intrinsic, &arguments, location, destination)?
        } else {
            self.execute_borrowed_intrinsic(intrinsic, &arguments, location, destination)?
        };
        let HostedOutcome::Complete(result) = outcome else {
            debug_assert!(matches!(outcome, HostedOutcome::Deferred));
            return Ok(());
        };
        if operands.a == NO_REGISTER {
            return Ok(());
        }
        let value = result.ok_or_else(|| {
            diagnostics::internal(
                self.executable.executable(),
                self.current_address,
                format!("Intrinsic {intrinsic:?} did not produce its verified result"),
            )
        })?;
        self.write(register(operands.a)?, value)
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
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        location: SourceLocation,
        destination: Option<fpas_bytecode::Register>,
    ) -> Result<HostedOutcome, VmError> {
        match self.execute_hosted_intrinsic(
            intrinsic,
            arguments,
            location,
            destination.map(|register| self.base + usize::from(register.get())),
        )? {
            HostedOutcome::Unhandled => {
                Ok(HostedOutcome::Complete(fpas_std::run_intrinsic_borrowed(
                    intrinsic,
                    arguments,
                    location,
                    self.layouts.as_ref(),
                )?))
            }
            outcome => Ok(outcome),
        }
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
