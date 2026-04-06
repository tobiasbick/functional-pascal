use super::super::diagnostics::VmError;
use super::super::{Worker, internal_error};
use fpas_bytecode::{Intrinsic, Op, SourceLocation};
use fpas_std::run_intrinsic;

mod callbacks;
mod console;
mod tui;

impl Worker {
    pub(super) fn try_exec_io(&mut self, op: Op, line: SourceLocation) -> Result<bool, VmError> {
        match op {
            Op::Print => {
                let value = self.pop(line)?;
                self.with_console(|c| c.write(&value, line))?;
                Ok(true)
            }
            Op::PrintLn => {
                let value = self.pop(line)?;
                self.with_console(|c| c.write_ln(&value, line))?;
                Ok(true)
            }
            Op::Intrinsic(id) => {
                let intrinsic = Intrinsic::from_u16(id).ok_or_else(|| {
                    internal_error(
                        format!("Unknown intrinsic opcode {id}"),
                        "This indicates a compiler/bytecode mismatch. Please report it.",
                        line,
                    )
                })?;

                if self.try_exec_console_intrinsic(intrinsic, line)? {
                    return Ok(true);
                }
                if self.try_exec_tui_intrinsic(intrinsic, line)? {
                    return Ok(true);
                }
                if self.try_exec_higher_order_intrinsic(intrinsic, line)? {
                    return Ok(true);
                }
                if self.try_exec_concurrency_intrinsic(intrinsic, line)? {
                    return Ok(true);
                }

                run_intrinsic(intrinsic, &mut self.stack, line)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
