use fpas_bytecode::{ChunkError, Op, SourceLocation, Value};

use crate::error::{CompileError, internal_compiler_error};

use super::Compiler;

pub(super) trait IntoEmitLocation {
    fn into_emit_location(self) -> SourceLocation;
}

impl IntoEmitLocation for SourceLocation {
    fn into_emit_location(self) -> SourceLocation {
        self
    }
}

impl IntoEmitLocation for (u32, u32) {
    fn into_emit_location(self) -> SourceLocation {
        SourceLocation::new(self.0, self.1)
    }
}

impl IntoEmitLocation for u32 {
    fn into_emit_location(self) -> SourceLocation {
        SourceLocation::new(self, 1)
    }
}

impl Compiler {
    pub(super) fn location_of(span: &fpas_lexer::Span) -> SourceLocation {
        SourceLocation::new(span.line, span.column)
    }

    pub(super) fn emit(&mut self, op: Op, location: impl IntoEmitLocation) -> usize {
        self.chunk.emit(op, location.into_emit_location())
    }

    pub(super) fn emit_constant(&mut self, value: Value, location: impl IntoEmitLocation) {
        let idx = self.chunk.add_constant(value);
        self.emit(Op::Constant(idx), location);
    }

    pub(super) fn patch_jump(
        &mut self,
        offset: usize,
        target: u32,
        location: impl IntoEmitLocation,
    ) -> Result<(), CompileError> {
        let location = location.into_emit_location();
        self.chunk
            .patch_jump(offset, target)
            .map_err(|error| match error {
                ChunkError::InvalidInstructionOffset { offset, code_len } => {
                    internal_compiler_error(
                        format!(
                            "Compiler produced an invalid jump patch offset {offset} (code length: {code_len})."
                        ),
                        "This is an internal compiler error. Re-run compilation and report the source program.",
                        location.line,
                        location.column,
                    )
                }
                ChunkError::NonJumpInstruction { offset, opcode } => internal_compiler_error(
                    format!(
                        "Compiler tried to patch instruction {offset}, but it is not a jump opcode ({opcode})."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    location.line,
                    location.column,
                ),
            })
    }
}
