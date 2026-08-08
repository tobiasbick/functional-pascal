//! Stack and register unit-object compilation adapters for the shared incremental engine.

use std::path::Path;

use fpas_unit::interface::UnitInterface;
use fpas_unit::object::{ChunkObject, RelocatableObject, encode_chunk_object, encode_object};
use fpas_unit::{
    Digest, ExpectedUnitIdentity, RegisterSidecarLoad, SidecarLoad, load_register_sidecar,
    load_sidecar,
};

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

pub(super) struct ChunkBackend;

impl UnitBackend for ChunkBackend {
    type Object = ChunkObject;

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
        encode_chunk_object(object).map_err(|error| BuildError::new(error.to_string()))
    }

    fn normalize(object: &mut Self::Object, source_id: u32) {
        for location in &mut object.locations {
            location.source_id = source_id;
        }
    }
}

pub(super) struct RegisterBackend;

impl UnitBackend for RegisterBackend {
    type Object = RelocatableObject;

    fn load(
        source_path: &Path,
        expected: &ExpectedUnitIdentity,
    ) -> Result<Option<ReusableObject<Self::Object>>, BuildError> {
        let loaded = load_register_sidecar(source_path, expected)
            .map_err(|error| BuildError::new(error.to_string()))?;
        Ok(match loaded {
            RegisterSidecarLoad::Reusable(loaded) => Some(ReusableObject {
                interface_hash: loaded.compiled.identity.interface_hash,
                object_hash: loaded.compiled.identity.object_hash,
                interface: loaded.interface,
                object: loaded.object,
            }),
            RegisterSidecarLoad::Missing
            | RegisterSidecarLoad::Stale(_)
            | RegisterSidecarLoad::Incompatible(_)
            | RegisterSidecarLoad::Corrupt(_) => None,
        })
    }

    fn compile(
        unit: &fpas_parser::Unit,
        direct_interfaces: &[UnitInterface],
        supporting_interfaces: &[UnitInterface],
    ) -> Result<(UnitInterface, Self::Object), Vec<fpas_compiler::CompileError>> {
        fpas_compiler::compile_register_unit_object_with_support(
            unit,
            direct_interfaces,
            supporting_interfaces,
        )
        .map(|compiled| (compiled.interface, compiled.object))
    }

    fn encode(object: &Self::Object) -> Result<Vec<u8>, BuildError> {
        encode_object(object).map_err(|error| BuildError::new(error.to_string()))
    }

    fn normalize(_object: &mut Self::Object, _source_id: u32) {}
}
