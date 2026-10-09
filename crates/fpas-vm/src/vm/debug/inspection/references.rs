//! Writable inspection origins for `var` parameters.
//! See `docs/pascal/language/functions/var-parameters.md` and `docs/pascal/tools/debugger.md`.

use std::sync::Arc;

use fpas_bytecode::{DebugTypeId, ReferenceRoot, ReferenceStep, VariableReference};

use super::targets::{MutationAccess, MutationPath, MutationRoot, MutationTarget};

/// Retains the referent's live storage instead of the parameter's reference register.
pub(super) fn mutation(
    executable: &fpas_bytecode::Executable,
    reference: &VariableReference,
    expected_type: DebugTypeId,
    generation: u32,
    frame_id: u64,
) -> MutationAccess {
    let root = match &reference.root {
        ReferenceRoot::Cell(cell) => MutationRoot::ClosureCell(Arc::clone(cell)),
        ReferenceRoot::Global(slot) => {
            if !executable
                .globals
                .get(*slot as usize)
                .is_some_and(|global| global.mutable)
            {
                return MutationAccess::NotMutable;
            }
            MutationRoot::Global(*slot as usize)
        }
    };
    let path = reference
        .path
        .iter()
        .map(|step| match step {
            ReferenceStep::Field(index) => Some(MutationPath::RecordField(*index as usize)),
            ReferenceStep::Element(index) => {
                usize::try_from(*index).ok().map(MutationPath::ArrayIndex)
            }
        })
        .collect::<Option<Vec<_>>>();
    let Some(path) = path else {
        return MutationAccess::Unavailable;
    };
    MutationAccess::Writable(MutationTarget {
        root,
        path,
        expected_type,
        generation,
        frame_id: Some(frame_id),
        initialized: true,
        initializer: None,
    })
}
