//! Concrete suffix types for erased generic record layouts.
//! See `docs/pascal/language/types/generics.md`.

use super::super::context::LoweringContext;
use crate::CompileError;
use fpas_ir::TypeId;
use fpas_lexer::Span;

impl LoweringContext {
    /// Return the semantic type of a field or index after generic substitution.
    pub(in crate::lowering) fn path_result_type(
        &mut self,
        span: Span,
        fallback: TypeId,
    ) -> Result<TypeId, CompileError> {
        match self.path_types.get(&(span.source_id, span.offset)).cloned() {
            Some(ty) => self.type_table.intern(&ty, span.line, span.column),
            None => Ok(fallback),
        }
    }

    /// Return a call suffix's result without confusing it with a callable field type.
    pub(in crate::lowering) fn postfix_result_type(
        &mut self,
        span: Span,
        fallback: TypeId,
    ) -> Result<TypeId, CompileError> {
        match self
            .postfix_types
            .get(&(span.source_id, span.offset))
            .cloned()
        {
            Some(ty) => self.type_table.intern(&ty, span.line, span.column),
            None => Ok(fallback),
        }
    }
}
