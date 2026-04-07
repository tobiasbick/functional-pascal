//! Emits VM opcodes and constants into the current chunk, with span-to-source mapping.
//!
//! **Documentation:** `docs/pascal/01-overview.md` (virtual machine model; from the repository root).

use fpas_bytecode::{ChunkError, Op, SourceLocation, Value};

use crate::error::{CompileError, compile_error, internal_compiler_error};
use fpas_diagnostics::codes::COMPILE_BYTECODE_OPERAND_OVERFLOW;
use fpas_lexer::Span;

use super::Compiler;

pub(super) trait IntoEmitLocation {
    fn into_emit_location(self) -> SourceLocation;
}

impl IntoEmitLocation for SourceLocation {
    fn into_emit_location(self) -> SourceLocation {
        self
    }
}

impl IntoEmitLocation for fpas_lexer::Span {
    fn into_emit_location(self) -> SourceLocation {
        SourceLocation::new_with_source(self.line, self.column, self.source_id)
    }
}

impl IntoEmitLocation for &fpas_lexer::Span {
    fn into_emit_location(self) -> SourceLocation {
        (*self).into_emit_location()
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
    fn span_at(location: SourceLocation) -> Span {
        Span {
            offset: 0,
            length: 0,
            line: location.line,
            column: location.column,
            source_id: location.source_id,
        }
    }

    pub(super) fn location_of(span: &fpas_lexer::Span) -> SourceLocation {
        SourceLocation::new_with_source(span.line, span.column, span.source_id)
    }

    pub(super) fn emit(&mut self, op: Op, location: impl IntoEmitLocation) -> usize {
        self.chunk.emit(op, location.into_emit_location())
    }

    pub(super) fn add_constant(
        &mut self,
        value: Value,
        location: impl IntoEmitLocation,
    ) -> Result<u16, CompileError> {
        let location = location.into_emit_location();
        self.chunk.add_constant(value).map_err(|error| match error {
            ChunkError::ConstantPoolOverflow { max_constants } => compile_error(
                COMPILE_BYTECODE_OPERAND_OVERFLOW,
                format!("Program uses more than {max_constants} constants"),
                format!(
                    "Reduce the number of distinct constants to at most {max_constants}."
                ),
                Self::span_at(location),
            ),
            other => internal_compiler_error(
                format!("Compiler failed to add a constant: {other}"),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                location.line,
                location.column,
            ),
        })
    }

    pub(super) fn emit_constant(
        &mut self,
        value: Value,
        location: impl IntoEmitLocation,
    ) -> Result<(), CompileError> {
        let location = location.into_emit_location();
        let idx = self.add_constant(value, location)?;
        self.emit(Op::Constant(idx), location);
        Ok(())
    }

    pub(super) fn checked_u8(
        count: usize,
        what: &str,
        span: fpas_lexer::Span,
    ) -> Result<u8, CompileError> {
        u8::try_from(count).map_err(|_| {
            compile_error(
                COMPILE_BYTECODE_OPERAND_OVERFLOW,
                format!("Too many {what} ({count}); maximum is {}", u8::MAX),
                format!("Reduce the number of {what} to at most {}.", u8::MAX),
                span,
            )
        })
    }

    pub(super) fn checked_u8_at(
        count: usize,
        what: &str,
        location: impl IntoEmitLocation,
    ) -> Result<u8, CompileError> {
        let location = location.into_emit_location();
        Self::checked_u8(count, what, Self::span_at(location))
    }

    pub(super) fn checked_u16(
        count: usize,
        what: &str,
        span: fpas_lexer::Span,
    ) -> Result<u16, CompileError> {
        u16::try_from(count).map_err(|_| {
            compile_error(
                COMPILE_BYTECODE_OPERAND_OVERFLOW,
                format!("Too many {what} ({count}); maximum is 65535"),
                format!("Reduce the number of {what} to at most 65535."),
                span,
            )
        })
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
                ChunkError::InvalidJumpTarget {
                    offset,
                    target,
                    code_len,
                } => internal_compiler_error(
                    format!(
                        "Compiler tried to patch jump {offset} to target {target}, but the current code length is {code_len}."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    location.line,
                    location.column,
                ),
                ChunkError::NonJumpInstruction { offset, opcode } => internal_compiler_error(
                    format!(
                        "Compiler tried to patch instruction {offset}, but it is not a jump opcode ({opcode})."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    location.line,
                    location.column,
                ),
                ChunkError::ConstantPoolOverflow { max_constants } => internal_compiler_error(
                    format!(
                        "Compiler hit a constant pool overflow while patching jumps (limit: {max_constants})."
                    ),
                    "This is an internal compiler error. Re-run compilation and report the source program.",
                    location.line,
                    location.column,
                ),
            })
    }
}
