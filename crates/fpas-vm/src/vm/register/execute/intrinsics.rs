//! Borrowed register-window execution for non-hosted standard-library intrinsics.

use fpas_bytecode::{AbcOperands, Intrinsic, NO_REGISTER, SourceLocation};

use super::scalar::register;
use crate::vm::register::hosted::HostedOutcome;
use crate::vm::register::worker::RegisterWorker;
use crate::vm::register::{VmError, diagnostics};

impl RegisterWorker {
    pub(in crate::vm::register) fn execute_intrinsic(
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
            .ok_or_else(|| self.intrinsic_window_error(operands))?;
        let location = self.intrinsic_location();
        let result = match self.execute_hosted_intrinsic(intrinsic, arguments, location)? {
            HostedOutcome::Complete(result) => result,
            HostedOutcome::Unhandled => {
                fpas_std::run_intrinsic_borrowed(intrinsic, arguments, location)?
            }
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
