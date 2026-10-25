//! Exact unsigned integers at the FPAS value boundary.

use crate::{CodecError, ErrorKind};

/// Full-width protocol counter, without a binary64 conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UInt64(u64);

impl UInt64 {
    /// Parse decimal ASCII digits; successful values have canonical decimal output.
    pub fn parse(text: &str) -> Result<Self, CodecError> {
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(CodecError::new(ErrorKind::InvalidInteger));
        }
        text.parse::<u64>()
            .map(Self)
            .map_err(|_| CodecError::new(ErrorKind::InvalidInteger))
    }

    /// Validate the canonical digits stored by a detached FPAS UInt64 record.
    pub fn canonical(text: &str) -> Result<Self, CodecError> {
        let value = Self::parse(text)?;
        if value.decimal() != text {
            return Err(CodecError::new(ErrorKind::InvalidInteger));
        }
        Ok(value)
    }

    /// Construct from a nonnegative FPAS integer.
    pub fn from_integer(value: i64) -> Result<Self, CodecError> {
        u64::try_from(value)
            .map(Self)
            .map_err(|_| CodecError::new(ErrorKind::InvalidInteger))
    }

    /// Return an FPAS integer only when the full value fits its signed range.
    pub fn to_integer(self) -> Result<i64, CodecError> {
        i64::try_from(self.0).map_err(|_| CodecError::new(ErrorKind::InvalidInteger))
    }

    /// Canonical decimal representation.
    #[must_use]
    pub fn decimal(self) -> String {
        self.0.to_string()
    }

    /// Native SDK counter with every bit preserved.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}
