//! Lowering for bound record method values (`C.Add`).
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_sema::BoundMethodInfo;

use super::super::Compiler;

impl Compiler {
    /// Emit a bound-method closure: capture the receiver already on the stack.
    ///
    /// Stack effect: `[..., receiver]` → `[..., Function]`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub(in crate::compiler) fn emit_bound_method_from_receiver(
        &mut self,
        info: &BoundMethodInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let thunk_name = format!(
            "$bound_{}_{}",
            canonical_thunk_key(&info.qualified_name),
            self.chunk.len()
        );
        let arity = info.visible_arity;
        let self_slot = u16::from(arity);
        let total_call_argc = Self::checked_u8_at(
            usize::from(arity) + 1,
            "bound method call arguments",
            location,
        )?;

        let jump_over = self.emit(Op::Jump(0), location);
        let code_start = self.chunk.len();
        self.chunk
            .insert_function(thunk_name.clone(), code_start, arity);

        // CallValue layout: args in slots 0..arity-1, Self capture in slot `arity`.
        self.emit(Op::GetLocal(self_slot), location);
        for arg_slot in 0..u16::from(arity) {
            self.emit(Op::GetLocal(arg_slot), location);
        }
        let method_idx = self.add_constant(Value::Str(info.qualified_name.clone()), location)?;
        self.emit(Op::Call(method_idx, total_call_argc), location);
        self.emit(Op::Return, location);

        let after = self.chunk.len() as u32;
        self.patch_jump(jump_over, after, location)?;

        // Receiver is already on the stack as the sole capture.
        let name_idx = self.add_constant(Value::Str(thunk_name), location)?;
        self.emit(Op::MakeClosure(name_idx, 1), location);
        Ok(())
    }
}

fn canonical_thunk_key(qualified: &str) -> String {
    qualified
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_ascii_lowercase()
}
