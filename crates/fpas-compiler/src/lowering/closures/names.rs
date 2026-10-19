//! Published anonymous routine names are independent of semantic AST lookup keys.
//! See `docs/pascal/tools/debugger-tracepoints.md`.

use fpas_ir::FunctionId;
use fpas_lexer::Span;

use super::ClosureRegistry;
use crate::CompileError;
use crate::lowering::context::unsupported;

impl ClosureRegistry<'_> {
    /// Discovery visits each owner's literal expressions in lexical source order.
    /// Nested bodies have their own owner, so they never advance the parent's ordinal.
    pub(super) fn next_closure_name(
        &mut self,
        owner: FunctionId,
        span: Span,
    ) -> Result<String, CompileError> {
        let ordinal = self.closure_ordinals.entry(owner).or_default();
        *ordinal = ordinal
            .checked_add(1)
            .ok_or_else(|| unsupported(span, "anonymous routine ordinal overflow"))?;
        // Program headings may equal a routine name. Root literals belong to the
        // reserved generated entry, whose name is also set in lowering/root.rs.
        let root_name = format!("$entry_{}", self.source_name.to_ascii_lowercase());
        let owner_name = self
            .callables
            .iter()
            .find(|(_, callable)| callable.function == owner)
            .map(|(name, _)| name.as_str())
            .or_else(|| {
                self.routines
                    .iter()
                    .find(|routine| routine.id == owner)
                    .map(|routine| routine.name.as_str())
            })
            .unwrap_or(&root_name);
        Ok(format!("{owner_name}.$closure_{ordinal}"))
    }
}
