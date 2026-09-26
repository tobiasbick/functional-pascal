//! Stable identities used to validate compiled units.

pub use fpas_binary::Digest;

/// Interface identity expected from one direct unit dependency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyIdentity {
    /// Canonical case-insensitive unit name.
    pub unit_name: String,
    /// Semantic interface hash consumed while compiling.
    pub interface_hash: Digest,
}

/// Identity recorded inside a compiled unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitIdentity {
    /// Canonical case-insensitive unit name.
    pub unit_name: String,
    /// Hash of the complete `.fpas` source bytes.
    pub source_hash: Digest,
    /// Hash of the exported semantic interface payload.
    pub interface_hash: Digest,
    /// Hash of the relocatable implementation payload.
    pub object_hash: Digest,
    /// Compiler build identity that emitted this object.
    pub compiler_version: String,
    /// Executable bytecode format expected by the object payload.
    pub bytecode_version: u32,
    /// Hash of all semantic and code-generation options.
    pub options_hash: Digest,
    /// Direct dependencies and the interface hashes used during compilation.
    pub dependencies: Vec<DependencyIdentity>,
}

/// Identity inputs known before a source unit is compiled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedUnitIdentity {
    /// Canonical case-insensitive unit name.
    pub unit_name: String,
    /// Hash of the current source bytes.
    pub source_hash: Digest,
    /// Current compiler build identity.
    pub compiler_version: String,
    /// Current executable bytecode format.
    pub bytecode_version: u32,
    /// Current semantic and code-generation option hash.
    pub options_hash: Digest,
    /// Current direct dependency interface hashes.
    pub dependencies: Vec<DependencyIdentity>,
}

/// Serialized semantic interface and relocatable object for one source unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledUnit {
    /// Validation identity and dependency fingerprints.
    pub identity: UnitIdentity,
    /// Versioned semantic interface payload.
    pub interface: Vec<u8>,
    /// Versioned relocatable bytecode object payload.
    pub object: Vec<u8>,
}
