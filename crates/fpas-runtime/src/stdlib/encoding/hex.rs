//! Canonical uppercase hexadecimal encoding and strict byte-pair decoding.

use super::{MAX_BYTES, error::CodecFailure};

pub(super) fn encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    let mut text = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        text.push(DIGITS[usize::from(byte >> 4)] as char);
        text.push(DIGITS[usize::from(byte & 15)] as char);
    }
    text
}

pub(super) fn decode(text: &str) -> Result<Vec<u8>, CodecFailure> {
    if text.len() > MAX_BYTES * 2 {
        return Err(CodecFailure::LimitExceeded);
    }
    if !text.len().is_multiple_of(2) {
        return Err(CodecFailure::InvalidLength { length: text.len() });
    }
    for (offset, byte) in text.bytes().enumerate() {
        digit(byte).ok_or(CodecFailure::InvalidCharacter { offset })?;
    }
    // No output buffer is allocated until the entire text has been validated.
    let mut output = Vec::with_capacity(text.len() / 2);
    for (index, pair) in text.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let offset = index * 2;
        let high = digit(pair[0]).ok_or(CodecFailure::InvalidCharacter { offset })?;
        let low = digit(pair[1]).ok_or(CodecFailure::InvalidCharacter { offset: offset + 1 })?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
