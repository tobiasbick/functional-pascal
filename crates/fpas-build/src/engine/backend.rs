//! Unit-object compilation adapter for the incremental engine.

use std::path::Path;

use fpas_diagnostics::codes::{BUILD_ARTIFACT_ENCODING_FAILED, BUILD_ARTIFACT_IO_FAILED};
use fpas_unit::interface::UnitInterface;
use fpas_unit::object::{RelocatableObject, encode_object};
use fpas_unit::{Digest, ExpectedUnitIdentity, SidecarError, SidecarLoad, load_sidecar};

use super::BuildError;

/// Classifies a compiled-unit sidecar failure as an encoding or filesystem error.
pub(super) fn sidecar_error(error: SidecarError, context: &str) -> BuildError {
    let code = match error {
        SidecarError::Format(_) => BUILD_ARTIFACT_ENCODING_FAILED,
        SidecarError::Io { .. } | SidecarError::LockTimeout(_) => BUILD_ARTIFACT_IO_FAILED,
    };
    BuildError::new(code, format!("{context}: {error}"))
}

pub(super) struct ReusableObject<Object> {
    pub(super) interface: UnitInterface,
    pub(super) object: Object,
    pub(super) interface_hash: Digest,
    pub(super) object_hash: Digest,
}

pub(super) trait UnitBackend {
    type Object;

    fn load(
        source_path: &Path,
        expected: &ExpectedUnitIdentity,
    ) -> Result<Option<ReusableObject<Self::Object>>, BuildError>;

    fn compile(
        unit: &fpas_parser::Unit,
        direct_interfaces: &[UnitInterface],
        supporting_interfaces: &[UnitInterface],
    ) -> Result<(UnitInterface, Self::Object), Vec<fpas_compiler::CompileError>>;

    fn encode(object: &Self::Object) -> Result<Vec<u8>, BuildError>;

    fn normalize(object: &mut Self::Object, source_id: u32);
}

pub(super) struct Backend;

impl UnitBackend for Backend {
    type Object = RelocatableObject;

    fn load(
        source_path: &Path,
        expected: &ExpectedUnitIdentity,
    ) -> Result<Option<ReusableObject<Self::Object>>, BuildError> {
        let loaded = load_sidecar(source_path, expected).map_err(|error| {
            sidecar_error(
                error,
                &format!(
                    "cannot load compiled unit beside `{}`",
                    source_path.display()
                ),
            )
        })?;
        Ok(match loaded {
            SidecarLoad::Reusable(loaded) => Some(ReusableObject {
                interface_hash: loaded.compiled.identity.interface_hash,
                object_hash: loaded.compiled.identity.object_hash,
                interface: loaded.interface,
                object: loaded.object,
            }),
            SidecarLoad::Missing
            | SidecarLoad::Stale(_)
            | SidecarLoad::Incompatible(_)
            | SidecarLoad::Corrupt(_) => None,
        })
    }

    fn compile(
        unit: &fpas_parser::Unit,
        direct_interfaces: &[UnitInterface],
        supporting_interfaces: &[UnitInterface],
    ) -> Result<(UnitInterface, Self::Object), Vec<fpas_compiler::CompileError>> {
        fpas_compiler::compile_unit_object_with_support(
            unit,
            direct_interfaces,
            supporting_interfaces,
        )
        .map(|compiled| (compiled.interface, compiled.object))
    }

    fn encode(object: &Self::Object) -> Result<Vec<u8>, BuildError> {
        encode_object(object).map_err(|error| {
            BuildError::new(
                BUILD_ARTIFACT_ENCODING_FAILED,
                format!("cannot encode compiled unit object: {error}"),
            )
        })
    }

    fn normalize(object: &mut Self::Object, source_id: u32) {
        object
            .sources
            .fill(crate::source_labels::source_label(source_id));
    }
}
