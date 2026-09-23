//! Deterministic pseudorandom state and integer range sampling.

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;

const REAL_SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);
const ORDERED_SIGN_BIT: u64 = 1_u64 << 63;

/// Pseudorandom state owned by one VM.
pub(in crate::vm::hosted) struct RandomState {
    rng: ChaCha12Rng,
    initialized: bool,
}

impl RandomState {
    /// Create state that seeds itself from the operating system on first use.
    pub(in crate::vm::hosted) fn new() -> Self {
        Self {
            rng: ChaCha12Rng::from_seed([0_u8; 32]),
            initialized: false,
        }
    }

    /// Replace the state with the portable sequence selected by `seed`.
    pub(super) fn set_seed(&mut self, seed: i64) {
        self.rng = ChaCha12Rng::from_seed(expand_seed(seed));
        self.initialized = true;
    }

    /// Replace the state with a seed from the operating system.
    pub(super) fn randomize(&mut self) -> Result<(), getrandom::Error> {
        let mut seed = [0_u8; 32];
        getrandom::fill(&mut seed)?;
        self.rng = ChaCha12Rng::from_seed(seed);
        self.initialized = true;
        Ok(())
    }

    /// Draw a real value in `[0.0, 1.0)`.
    pub(super) fn real(&mut self) -> Result<f64, getrandom::Error> {
        let bits = self.rng()?.next_u64() >> 11;
        Ok((bits as f64) * REAL_SCALE)
    }

    /// Draw an unbiased integer from the inclusive range.
    pub(super) fn integer(&mut self, lo: i64, hi: i64) -> Result<i64, getrandom::Error> {
        let rng = self.rng()?;
        uniform_i64_with(lo, hi, || Ok(rng.next_u64()))
    }

    fn rng(&mut self) -> Result<&mut ChaCha12Rng, getrandom::Error> {
        if !self.initialized {
            self.randomize()?;
        }
        Ok(&mut self.rng)
    }
}

/// Draw an unbiased integer from an inclusive range using fallible `u64` samples.
pub(in crate::vm::hosted) fn uniform_i64_with<E>(
    lo: i64,
    hi: i64,
    mut next: impl FnMut() -> Result<u64, E>,
) -> Result<i64, E> {
    let ordered_lo = (lo as u64) ^ ORDERED_SIGN_BIT;
    let ordered_hi = (hi as u64) ^ ORDERED_SIGN_BIT;
    let width = ordered_hi.wrapping_sub(ordered_lo).wrapping_add(1);
    let offset = if width == 0 {
        next()?
    } else {
        let rejection_floor = width.wrapping_neg() % width;
        loop {
            let sample = next()?;
            if sample >= rejection_floor {
                break sample % width;
            }
        }
    };
    Ok(((ordered_lo.wrapping_add(offset)) ^ ORDERED_SIGN_BIT) as i64)
}

fn expand_seed(seed: i64) -> [u8; 32] {
    let mut state = seed as u64;
    let mut expanded = [0_u8; 32];
    for chunk in expanded.as_chunks_mut::<8>().0 {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^= value >> 31;
        chunk.copy_from_slice(&value.to_le_bytes());
    }
    expanded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_seed_replays_the_same_sequence() {
        let mut first = RandomState::new();
        let mut second = RandomState::new();
        first.set_seed(-17);
        second.set_seed(-17);
        let first_values = [
            first.integer(i64::MIN, i64::MAX).unwrap(),
            first.integer(1, 6).unwrap(),
        ];
        let second_values = [
            second.integer(i64::MIN, i64::MAX).unwrap(),
            second.integer(1, 6).unwrap(),
        ];
        assert_eq!(first_values, second_values);
        assert_eq!(first.real().unwrap(), second.real().unwrap());
    }

    #[test]
    fn seeded_sequence_keeps_its_portable_contract() {
        let mut state = RandomState::new();
        state.set_seed(-17);
        assert_eq!(state.integer(-1_000_000, 1_000_000), Ok(436_834));
        assert_eq!(state.real(), Ok(0.018_693_080_316_887_545));
        assert_eq!(state.integer(-1_000_000, 1_000_000), Ok(-809_256));
    }

    #[test]
    fn inclusive_sampler_handles_equal_and_full_width_ranges() {
        assert_eq!(uniform_i64_with(7, 7, || Ok::<_, ()>(u64::MAX)), Ok(7));
        assert_eq!(
            uniform_i64_with(i64::MIN, i64::MAX, || Ok::<_, ()>(u64::MAX)),
            Ok(i64::MAX)
        );
    }

    #[test]
    fn rejection_sampler_discards_biased_prefix() {
        let mut samples = [0_u64, 7].into_iter();
        let actual = uniform_i64_with(10, 15, || Ok::<_, ()>(samples.next().unwrap()));
        assert_eq!(actual, Ok(11));
    }
}
