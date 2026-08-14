//! Task-result type specialization for debugger metadata.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_ir::{IrType, TypeId};
use fpas_sema::Ty;

use super::{DYNAMIC, TypeTable};
use crate::CompileError;

impl TypeTable {
    /// Prefer a concrete `go` result type over a bare `task` annotation.
    ///
    /// Semantic analysis already stores the spawned result type on the binding;
    /// debugger assignment compares those portable result types structurally.
    pub fn specialize_task_binding(&self, declared: TypeId, inferred: TypeId) -> TypeId {
        match (self.kind(declared), self.kind(inferred)) {
            (Some(IrType::Task(declared_inner)), Some(IrType::Task(actual_inner)))
                if *declared_inner == DYNAMIC && *actual_inner != DYNAMIC =>
            {
                inferred
            }
            _ => declared,
        }
    }

    /// Whether a declared binding needs its bare task result type specialized.
    pub fn is_bare_task_binding(&self, declared: TypeId) -> bool {
        matches!(self.kind(declared), Some(IrType::Task(inner)) if *inner == DYNAMIC)
    }

    /// Intern a semantic initializer type, then specialize a bare `task` binding.
    pub fn specialize_task_binding_from_sema(
        &mut self,
        declared: TypeId,
        inferred: Option<&Ty>,
        line: u32,
        column: u32,
    ) -> Result<TypeId, CompileError> {
        if !self.is_bare_task_binding(declared) {
            return Ok(declared);
        }
        let Some(ty) = inferred else {
            return Ok(declared);
        };
        if matches!(ty, Ty::Error | Ty::Named(_)) {
            return Ok(declared);
        }
        let inferred = self.intern(ty, line, column)?;
        Ok(self.specialize_task_binding(declared, inferred))
    }

    /// Intern `task of inner`, reusing an existing type identifier when present.
    pub fn intern_task_type(
        &mut self,
        inner: TypeId,
        span: fpas_lexer::Span,
    ) -> Result<TypeId, CompileError> {
        self.intern_kind(IrType::Task(inner), span)
    }
}
