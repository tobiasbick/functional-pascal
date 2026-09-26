//! BLAKE3 content digests.

use std::fmt;

/// BLAKE3 digest used for source, option, interface, and payload identities.
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
