//! Task handle type interning for lowering and debugger metadata.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-handles.md`

use super::TypeTable;
use crate::CompileError;
use fpas_ir::{IrType, TypeId};

impl TypeTable {
    /// Intern `task of inner`, reusing an existing type identifier when present.
    pub fn intern_task_type(
        &mut self,
        inner: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Task(inner), span)
    }
}
