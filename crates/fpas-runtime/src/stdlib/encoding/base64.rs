//! Strict RFC 4648 Base64 and unpadded Base64url over byte slices.

use super::MAX_BYTES;
use super::error::CodecFailure;

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(super) fn encode(bytes: &[u8], url: bool) -> String {
    let alphabet = if url { URL } else { STANDARD };
    let capacity = if url {
        bytes.len() / 3 * 4 + (bytes.len() % 3 * 8).div_ceil(6)
    } else {
        bytes.len().div_ceil(3) * 4
    };
    let mut output = String::with_capacity(capacity);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(alphabet[(first >> 2) as usize] as char);
        output.push(alphabet[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(alphabet[(((second & 15) << 2) | (third >> 6)) as usize] as char);
        } else if !url {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(alphabet[(third & 63) as usize] as char);
        } else if !url {
            output.push('=');
        }
    }
    output
}

pub(super) fn decode(text: &str, url: bool) -> Result<Vec<u8>, CodecFailure> {
    let bytes = text.as_bytes();
    check_limits(bytes, url)?;
    let data_length = validate(bytes, url)?;
    let output_length = decoded_length(data_length);
    let mut output = Vec::with_capacity(output_length);
    let mut accumulator = 0_u16;
    let mut bits = 0_u32;
    for (offset, byte) in bytes[..data_length].iter().copied().enumerate() {
        // Validation finished before allocation, so this lookup always succeeds.
        let digit = sextet(byte, url).ok_or(CodecFailure::InvalidCharacter { offset })?;
        accumulator = (accumulator << 6) | u16::from(digit);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((accumulator >> bits) as u8);
            accumulator &= (1 << bits) - 1;
        }
    }
    Ok(output)
}

fn check_limits(bytes: &[u8], url: bool) -> Result<(), CodecFailure> {
    let max_length = if url {
        MAX_BYTES / 3 * 4 + (MAX_BYTES % 3 * 8).div_ceil(6)
    } else {
        MAX_BYTES.div_ceil(3) * 4
    };
    if bytes.len() > max_length {
        return Err(CodecFailure::LimitExceeded);
    }
    let padding = if url {
        0
    } else {
        bytes
            .iter()
            .rev()
            .take(2)
            .take_while(|byte| **byte == b'=')
            .count()
    };
    if decoded_length(bytes.len() - padding) > MAX_BYTES {
        return Err(CodecFailure::LimitExceeded);
    }
    Ok(())
}

fn validate(bytes: &[u8], url: bool) -> Result<usize, CodecFailure> {
    let mut padding = None;
    for (offset, byte) in bytes.iter().copied().enumerate() {
        if byte == b'=' {
            if url {
                return Err(CodecFailure::InvalidPadding { offset });
            }
            padding.get_or_insert(offset);
        } else if let Some(offset) = padding {
            return Err(CodecFailure::InvalidPadding { offset });
        } else if sextet(byte, url).is_none() {
            return Err(CodecFailure::InvalidCharacter { offset });
        }
    }
    if (!url && !bytes.len().is_multiple_of(4)) || (url && bytes.len() % 4 == 1) {
        return Err(CodecFailure::InvalidLength {
            length: bytes.len(),
        });
    }
    let data_length = padding.unwrap_or(bytes.len());
    if bytes.len() - data_length > 2 {
        return Err(CodecFailure::InvalidPadding {
            offset: data_length,
        });
    }
    let mask = match data_length % 4 {
        2 => 15,
        3 => 3,
        _ => return Ok(data_length),
    };
    let offset = data_length - 1;
    let digit = sextet(bytes[offset], url).ok_or(CodecFailure::InvalidCharacter { offset })?;
    if digit & mask != 0 {
        return Err(CodecFailure::NonCanonical { offset });
    }
    Ok(data_length)
}

fn decoded_length(data_length: usize) -> usize {
    data_length / 4 * 3 + data_length % 4 * 6 / 8
}

fn sextet(byte: u8, url: bool) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' if !url => Some(62),
        b'/' if !url => Some(63),
        b'-' if url => Some(62),
        b'_' if url => Some(63),
        _ => None,
    }
}
