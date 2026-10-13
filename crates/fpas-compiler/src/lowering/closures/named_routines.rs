//! Named helpers in anonymous callable scopes reuse named-routine collection and lowering.
//! **Documentation:** `docs/pascal/language/functions/nested.md`.

use super::ClosureRegistry;
use crate::CompileError;
use crate::lowering::context::{CaptureInput, unsupported};
use crate::lowering::{routines, types};
use fpas_ir::FunctionId;
use fpas_parser::Decl;
use fpas_sema::AnalysisMetadata;

/// One named routine collected after its anonymous lexical owner is allocated.
pub(in crate::lowering) struct NamedClosureRoutine<'a> {
    /// Source body lowered by the ordinary routine implementation.
    pub routine: routines::Routine<'a>,
    /// Allocated callable identifier.
    pub id: FunctionId,
    /// Lexically qualified runtime name.
    pub name: String,
    /// Anonymous or named lexical parent.
    pub owner: FunctionId,
}

impl<'a> ClosureRegistry<'a> {
    /// Collect all sibling signatures before discovering helper bodies and their closures.
    pub(super) fn collect_named_helpers(
        &mut self,
        declarations: &'a [Decl],
        owner: FunctionId,
        owner_name: &str,
        captures: &[CaptureInput],
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let first_id = self.next_id;
        let mut sources = Vec::new();
        let mut owners = Vec::new();
        let mut names = Vec::new();
        routines::collect(
            declarations,
            &mut sources,
            &mut owners,
            &mut names,
            owner,
            owner_name,
            first_id.saturating_sub(1) as usize,
        );
        if sources.is_empty() {
            return Ok(());
        }
        let span = match &declarations[0] {
            Decl::Function(value) => value.span,
            Decl::Procedure(value) => value.span,
            Decl::Const(value) => value.span,
            Decl::Var(value) => value.span,
            Decl::TypeDef(value) => value.span,
        };
        let table = routines::callable_table(
            &sources, &owners, &names, types, metadata, first_id, owner, captures,
        )?;
        self.next_id = first_id
            .checked_add(
                u32::try_from(sources.len())
                    .map_err(|_| unsupported(span, "function identifier overflow"))?,
            )
            .ok_or_else(|| unsupported(span, "function identifier overflow"))?;
        self.seed_named_nested_cells(&owners, &names, &table);
        self.callables.extend(table);
        for (index, source) in sources.into_iter().enumerate() {
            let id = FunctionId::new(first_id + index as u32);
            self.discover_statements(source.statements(), id, metadata, types)?;
            self.named_routines.push(NamedClosureRoutine {
                routine: source,
                id,
                name: names[index].clone(),
                owner: owners[index],
            });
        }
        Ok(())
    }
}
