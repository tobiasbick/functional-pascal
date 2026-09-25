//! Sampled scalar-to-byte offsets that bound character indexing in non-ASCII strings.

/// Unicode scalars between two sampled byte offsets.
pub(super) const STRIDE: usize = 64;

/// Byte offsets of every [`STRIDE`]-th Unicode scalar, starting with scalar 0.
pub(super) fn sample(text: &str) -> Box<[usize]> {
    text.char_indices()
        .step_by(STRIDE)
        .map(|(offset, _)| offset)
        .collect()
}

/// Byte offset of scalar `index`, walking fewer than [`STRIDE`] scalars from its sample.
///
/// `index` must be below the scalar count of `text`.
pub(super) fn byte_offset(text: &str, samples: &[usize], index: usize) -> Option<usize> {
    let start = *samples.get(index / STRIDE)?;
    text[start..]
        .char_indices()
        .nth(index % STRIDE)
        .map(|(offset, _)| start + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_offsets_match_a_linear_scan() {
        let text = "aé日😀".repeat(50);
        let samples = sample(&text);
        for (index, (expected, _)) in text.char_indices().enumerate() {
            assert_eq!(byte_offset(&text, &samples, index), Some(expected));
        }
        assert_eq!(byte_offset(&text, &samples, text.chars().count()), None);
    }
}
