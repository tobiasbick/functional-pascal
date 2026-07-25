//! Lowering for property setter assignment (`B.Text := …`).
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, Expr};
use fpas_sema::PropertyWriteInfo;

use super::super::Compiler;

impl Compiler {
    /// Compile a property assignment: evaluate receiver once, value once, call setter.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub(in crate::compiler) fn compile_property_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        info: &PropertyWriteInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        target
            .parts
            .get(..info.receiver_part_count)
            .ok_or_else(|| {
                internal_compiler_error(
                    "Property-write receiver metadata exceeds the designator path.",
                    "Re-run compilation and report this internal compiler error.",
                    target.span.line,
                    target.span.column,
                )
            })?;
        self.compile_property_receiver_prefix(
            target,
            info.receiver_part_count,
            &info.receiver_reads,
        )?;
        self.compile_expr(value)?;
        let setter = self.qualify_name(&info.setter_name).to_string();
        let name_idx = self.add_constant(Value::Str(setter.into()), location)?;
        self.emit(Op::Call(name_idx, 2), location);
        self.emit(Op::Pop, location);
        Ok(())
    }
}
