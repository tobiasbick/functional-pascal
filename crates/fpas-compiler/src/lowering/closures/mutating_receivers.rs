//! Storage discovery for writable intrinsic receivers with field or element paths.
//!
//! **Documentation:** `docs/pascal/language/types/array/mutating.md`

use fpas_ir::FunctionId;
use fpas_parser::{Designator, DesignatorPart};
use fpas_sema::{AnalysisMetadata, ParamMode, intrinsic_std_receiver_mode};

use super::ClosureRegistry;

impl ClosureRegistry<'_> {
    /// Makes a receiver root addressable when its path needs a reference.
    /// Simple local arrays retain their direct push/pop operations.
    pub(super) fn register_mutating_receiver(
        &mut self,
        designator: &Designator,
        key: usize,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
    ) {
        if designator.parts.len() <= 2 {
            return;
        }
        if metadata
            .fluent_calls
            .get(&key)
            .is_some_and(|target| intrinsic_std_receiver_mode(&target.name) == ParamMode::Var)
            && let Some(DesignatorPart::Ident(root, _)) = designator.parts.first()
        {
            self.cell_names
                .entry(owner)
                .or_default()
                .insert(root.to_ascii_lowercase());
        }
    }
}
