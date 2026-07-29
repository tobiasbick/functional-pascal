//! Stable identities recorded in compiled program images.

use std::fmt;

/// BLAKE3 digest used for program source, options, units, and payloads.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest([u8; Self::LENGTH]);

impl Digest {
    /// Digest length in bytes.
    pub const LENGTH: usize = 32;

    /// Hash arbitrary bytes with BLAKE3.
    #[must_use]
    pub fn of(bytes: impl AsRef<[u8]>) -> Self {
        Self(*blake3::hash(bytes.as_ref()).as_bytes())
    }

    /// Create a digest from its stable byte representation.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Return the stable byte representation.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

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
