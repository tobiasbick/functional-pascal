//! Anonymous routine names retain the lexical path used by ordinary call resolution.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`.

use fpas_ir::FunctionId;
use fpas_lexer::Span;

use super::ClosureRegistry;
use crate::CompileError;
use crate::lowering::context::unsupported;

impl ClosureRegistry<'_> {
    pub(super) fn lexical_closure_name(
        &self,
        owner: FunctionId,
        synthetic_name: &str,
        span: Span,
    ) -> Result<String, CompileError> {
        let parent = if owner == FunctionId::new(0) {
            return Ok(synthetic_name.to_owned());
        } else if let Some(routine) = self.routines.iter().find(|routine| routine.id == owner) {
            routine.name.as_str()
        } else {
            self.callables
                .iter()
                .find(|(_, callable)| callable.function == owner)
                .map(|(name, _)| name.as_str())
                .ok_or_else(|| unsupported(span, "closure without a lexical owner"))?
        };
        Ok(format!("{parent}.{synthetic_name}"))
    }
}
