//! Unit-object compilation adapter for the incremental engine.

use std::path::Path;

use fpas_unit::interface::UnitInterface;
use fpas_unit::object::{RelocatableObject, encode_object};
use fpas_unit::{Digest, ExpectedUnitIdentity, SidecarLoad, load_sidecar};

use super::BuildError;

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
        let loaded = load_sidecar(source_path, expected)
            .map_err(|error| BuildError::new(error.to_string()))?;
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
        encode_object(object).map_err(|error| BuildError::new(error.to_string()))
    }

    fn normalize(object: &mut Self::Object, source_id: u32) {
        let source = format!("source-{source_id}.fpas");
        object.sources.fill(source);
    }
}
