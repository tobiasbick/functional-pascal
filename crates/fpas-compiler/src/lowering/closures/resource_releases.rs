//! Callable wrappers for native resource-release operations.

use fpas_ir::FunctionId;
use fpas_parser::LocalBinding;
use fpas_sema::{AnalysisMetadata, NativeCallable};

use super::ClosureRegistry;
use crate::CompileError;
use crate::lowering::types;

impl ClosureRegistry<'_> {
    pub(super) fn register_resource_release(
        &mut self,
        binding: &LocalBinding,
        owner: FunctionId,
        metadata: &AnalysisMetadata,
        types: &mut types::TypeTable,
    ) -> Result<(), CompileError> {
        let key = (binding.span.source_id, binding.span.offset);
        let Some(release) = metadata.resource_releases.get(&key) else {
            return Ok(());
        };
        if !release.native {
            return Ok(());
        }
        let info = NativeCallable {
            name: release.name.clone(),
            receiver_ty: None,
            receiver_part_count: 0,
        };
        let before = self.intrinsic_routines.len();
        self.register_native_value(
            binding.span.offset,
            &info,
            &release.signature,
            owner,
            binding.span,
            types,
        )?;
        if self.intrinsic_routines.len() > before
            && let Some(routine) = self.intrinsic_routines.last_mut()
        {
            routine.name = format!("$resource_release_{}_{}", release.name, routine.id.get());
        }
        Ok(())
    }
}
