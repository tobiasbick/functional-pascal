//! Direct tail calls that reuse the current frame.

use super::*;

impl Worker {
    /// Replace the current frame with a direct callee whose result this frame would return.
    ///
    /// Arguments move to the frame base in ascending order; the argument window never sits below
    /// its destination, so no argument is overwritten before it moves. The saved caller frame and
    /// its return destination stay as they are, so the callee returns straight to them.
    pub(in crate::vm) fn tail_call(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let target = FunctionId::new(operands.b);
        let image = self.executable.executable();
        let info = image
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    image,
                    self.current_address,
                    "Tail-call target is outside the function table",
                )
            })?;
        let count = usize::from(operands.auxiliary);
        if count != usize::from(info.arity) || info.capture_count != 0 {
            return Err(diagnostics::internal(
                image,
                self.current_address,
                format!(
                    "Tail-call signature mismatch: expected {} arguments and no captures, got {count} arguments and {} captures",
                    info.arity, info.capture_count
                ),
            ));
        }
        let frame_size = usize::from(info.register_count);
        let entry = usize::try_from(info.code.start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                self.current_address,
                "Tail-call address does not fit this host",
            )
        })?;
        let arguments = self.base + usize::from(operands.c);
        for offset in 0..count {
            let value = self.take_register(arguments + offset)?;
            self.store_register(self.base + offset, value)?;
        }
        let frame_end = self
            .base
            .checked_add(frame_size)
            .filter(|end| *end <= MAX_REGISTER_SLOTS)
            .ok_or_else(|| {
                diagnostics::at_address(
                    self.executable.executable(),
                    self.current_address,
                    RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                    "Register stack overflow",
                    "Reduce the number of live registers per function.",
                )
            })?;
        self.release_registers(self.base + count);
        self.activate_registers(frame_end);
        self.function = target;
        self.ip = entry;
        Ok(())
    }
}
