//! Semantic interface hashing.

use crate::Digest;

use super::{InterfaceFormatError, UnitInterface, encode_interface};

impl UnitInterface {
    /// Hash the canonical serialized interface.
    pub fn digest(&self) -> Result<Digest, InterfaceFormatError> {
        encode_interface(self).map(|bytes| Digest::of(&bytes))
    }
}
