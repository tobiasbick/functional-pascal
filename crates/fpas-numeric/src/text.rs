//! Numeric literal bodies and signed input text use one grammar and range policy.
//!
//! **Documentation:** `docs/pascal/language/basics/number-literals.md` and
//! `docs/pascal/std/text/parse.md`.

use crate::{NumericError, real};

/// Maximum lexical integer magnitude, valid only with a negative sign.
pub const MINIMUM_INTEGER_MAGNITUDE: u64 = 1_u64 << 63;

/// Parses an unsigned decimal or `$` hexadecimal literal body with digit separators.
pub fn integer_magnitude(text: &str) -> Option<u64> {
    let (digits, radix) = text
        .strip_prefix('$')
        .map_or((text, 10), |digits| (digits, 16));
    let mut index = 0;
    let normalized = digit_run(digits.as_bytes(), &mut index, radix)?;
    if index != digits.len() {
        return None;
    }
    let value = u64::from_str_radix(&normalized, radix).ok()?;
    (value <= MINIMUM_INTEGER_MAGNITUDE).then_some(value)
}

/// Applies a sign to a lexical magnitude without overflowing at the integer minimum.
pub fn signed_integer(magnitude: u64, negative: bool) -> Option<i64> {
    if negative && magnitude == MINIMUM_INTEGER_MAGNITUDE {
        Some(i64::MIN)
    } else {
        let value = i64::try_from(magnitude).ok()?;
        Some(if negative { -value } else { value })
    }
}

/// Parses signed integer input after trimming surrounding whitespace.
pub fn integer(text: &str) -> Option<i64> {
    let (body, negative) = signed_body(text.trim());
    signed_integer(integer_magnitude(body)?, negative)
}

/// Parses the signed hexadecimal text accepted by Parse.HexInt, with an optional $, 0x, or 0X prefix.
pub fn hex_integer(text: &str) -> Option<i64> {
    let (body, negative) = signed_body(text.trim());
    let digits = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
        .or_else(|| body.strip_prefix('$'))
        .unwrap_or(body);
    signed_integer(integer_magnitude(&format!("${digits}"))?, negative)
}

/// Parses an unsigned real literal body, requiring a fractional part and a finite result.
pub fn real_body(text: &str) -> Option<f64> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let whole = digit_run(bytes, &mut index, 10)?;
    if bytes.get(index) != Some(&b'.') {
        return None;
    }
    index += 1;
    let fraction = digit_run(bytes, &mut index, 10)?;
    let mut normalized = format!("{whole}.{fraction}");
    if bytes
        .get(index)
        .is_some_and(|byte| matches!(byte, b'e' | b'E'))
    {
        normalized.push('e');
        index += 1;
        if bytes
            .get(index)
            .is_some_and(|byte| matches!(byte, b'+' | b'-'))
        {
            normalized.push(bytes[index] as char);
            index += 1;
        }
        normalized.push_str(&digit_run(bytes, &mut index, 10)?);
    }
    if index != bytes.len() {
        return None;
    }
    let value = real::finite(normalized.parse::<f64>().ok()?).ok()?;
    let nonzero = whole
        .bytes()
        .chain(fraction.bytes())
        .any(|byte| byte != b'0');
    if value == 0.0 && nonzero {
        None
    } else {
        Some(value)
    }
}

/// Parses real input with an optional sign and surrounding whitespace, preserving -0.0.
pub fn real(text: &str) -> Option<f64> {
    let (body, negative) = signed_body(text.trim());
    let value = real_body(body)?;
    Some(if negative { -value } else { value })
}

/// Reports whether input has either of the shared integer or real text forms.
pub fn is_numeric(text: &str) -> bool {
    integer(text).is_some() || real(text).is_some()
}

/// Formats a finite real with a fractional part and a bit-preserving parsing round-trip.
pub fn format_real(value: f64) -> Result<String, NumericError> {
    real::finite(value)?;
    let mut output = value.to_string();
    if !output.contains('.') {
        if let Some(index) = output.find(['e', 'E']) {
            output.insert_str(index, ".0");
        } else {
            output.push_str(".0");
        }
    }
    Ok(output)
}

fn signed_body(text: &str) -> (&str, bool) {
    if let Some(body) = text.strip_prefix('-') {
        (body, true)
    } else {
        (text.strip_prefix('+').unwrap_or(text), false)
    }
}

fn digit_run(bytes: &[u8], index: &mut usize, radix: u32) -> Option<String> {
    let mut output = String::new();
    let digit = |byte: u8| (byte as char).is_digit(radix);
    while let Some(&byte) = bytes.get(*index) {
        if digit(byte) {
            output.push(byte as char);
            *index += 1;
        } else if byte == b'_'
            && !output.is_empty()
            && bytes.get(*index + 1).is_some_and(|byte| digit(*byte))
        {
            *index += 1;
            output.push(bytes[*index] as char);
            *index += 1;
        } else {
            break;
        }
    }
    (!output.is_empty()).then_some(output)
}
