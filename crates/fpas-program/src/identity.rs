//! Stable identities recorded in compiled program images.

pub use fpas_binary::Digest;

/// Linked implementation identity for one reachable unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedUnitIdentity {
    /// Canonical case-insensitive unit name.
    pub unit_name: String,
    /// Hash of the relocatable implementation payload used while linking.
    pub object_hash: Digest,
}

/// Compatibility and input identity recorded in a compiled program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIdentity {
    /// Compiler build identity that emitted the image.
    pub compiler_version: String,
    /// Executable bytecode version expected by the image.
    pub bytecode_version: u32,
    /// Hash of the main program source bytes.
    pub source_hash: Digest,
    /// Hash of semantic and code-generation options.
    pub options_hash: Digest,
    /// Reachable units in deterministic link order.
    pub units: Vec<LinkedUnitIdentity>,
}
