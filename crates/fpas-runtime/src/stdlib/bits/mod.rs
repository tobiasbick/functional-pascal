//! Signed 64-bit bit operations for `Std.Bits`.
//!
//! Documentation: `docs/pascal/std/numeric/bits.md`.

mod intrinsic;

pub(crate) use intrinsic::run;

/// Returns the bitwise intersection of two signed 64-bit patterns.
#[must_use]
pub const fn bit_and(left: i64, right: i64) -> i64 {
    left & right
}

/// Returns the bitwise union of two signed 64-bit patterns.
#[must_use]
pub const fn bit_or(left: i64, right: i64) -> i64 {
    left | right
}

/// Returns the bits that differ between two signed 64-bit patterns.
#[must_use]
pub const fn bit_xor(left: i64, right: i64) -> i64 {
    left ^ right
}

/// Complements all 64 bits, retaining the signed interpretation of the result.
#[must_use]
pub const fn bit_not(value: i64) -> i64 {
    !value
}

/// Shifts left, discarding high bits; returns `None` for counts outside `0..63`.
#[must_use]
pub fn shift_left(value: i64, count: i64) -> Option<i64> {
    Some(value.wrapping_shl(shift_count(count)?))
}

/// Shifts right with sign extension; returns `None` for counts outside `0..63`.
#[must_use]
pub fn shift_right(value: i64, count: i64) -> Option<i64> {
    Some(value.wrapping_shr(shift_count(count)?))
}

fn shift_count(count: i64) -> Option<u32> {
    u32::try_from(count).ok().filter(|count| *count < 64)
}

#[cfg(test)]
mod tests;
