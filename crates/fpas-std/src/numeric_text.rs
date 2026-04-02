//! Helpers for parsing Pascal numeric text used by `Std.Conv` and `Std.Str`.
//!
//! **Documentation:** `docs/pascal/02-basics.md`, `docs/pascal/std/conv.md`,
//! and `docs/pascal/std/str.md` (from the repository root).

/// Returns `true` when `text` is a valid Pascal integer or real literal after trimming.
pub(crate) fn is_pascal_numeric(text: &str) -> bool {
    parse_pascal_integer(text).is_some() || parse_pascal_real(text).is_some()
}

/// Parses a Pascal real literal string after trimming whitespace.
pub(crate) fn parse_pascal_real(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let bytes = trimmed.as_bytes();
    let mut index = 0;
    let mut normalized = String::with_capacity(trimmed.len());

    if bytes.is_empty() {
        return None;
    }

    if matches!(bytes[index], b'+' | b'-') {
        normalized.push(bytes[index] as char);
        index += 1;
    }

    let int_digits = consume_digit_run(bytes, &mut index)?;
    normalized.push_str(&int_digits);

    if index >= bytes.len() || bytes[index] != b'.' {
        return None;
    }
    normalized.push('.');
    index += 1;

    let frac_digits = consume_digit_run(bytes, &mut index)?;
    normalized.push_str(&frac_digits);

    if index < bytes.len() && matches!(bytes[index], b'e' | b'E') {
        normalized.push('e');
        index += 1;

        if index < bytes.len() && matches!(bytes[index], b'+' | b'-') {
            normalized.push(bytes[index] as char);
            index += 1;
        }

        let exponent_digits = consume_digit_run(bytes, &mut index)?;
        normalized.push_str(&exponent_digits);
    }

    if index != bytes.len() {
        return None;
    }

    let value = normalized.parse::<f64>().ok()?;
    value.is_finite().then_some(value)
}

fn parse_pascal_integer(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    let bytes = trimmed.as_bytes();
    let mut index = 0;
    let mut normalized = String::with_capacity(trimmed.len());

    if bytes.is_empty() {
        return None;
    }

    if matches!(bytes[index], b'+' | b'-') {
        normalized.push(bytes[index] as char);
        index += 1;
    }

    let digits = consume_digit_run(bytes, &mut index)?;
    if index != bytes.len() {
        return None;
    }

    normalized.push_str(&digits);
    normalized.parse::<i64>().ok()
}

fn consume_digit_run(bytes: &[u8], index: &mut usize) -> Option<String> {
    let mut digits = String::new();
    let mut saw_digit = false;

    while *index < bytes.len() {
        let current = bytes[*index];
        if current.is_ascii_digit() {
            digits.push(current as char);
            *index += 1;
            saw_digit = true;
            continue;
        }

        if current == b'_' && saw_digit {
            let next = bytes.get(*index + 1).copied();
            if next.is_some_and(|candidate| candidate.is_ascii_digit()) {
                *index += 1;
                continue;
            }
        }

        break;
    }

    saw_digit.then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::{is_pascal_numeric, parse_pascal_real};

    #[test]
    fn parses_pascal_real_literals_only() {
        assert_eq!(parse_pascal_real("3.14"), Some(3.14));
        assert_eq!(parse_pascal_real("-1.5e2"), Some(-150.0));
        assert_eq!(parse_pascal_real("1_000.5_0"), Some(1000.50));
        assert_eq!(parse_pascal_real(".5"), None);
        assert_eq!(parse_pascal_real("5."), None);
        assert_eq!(parse_pascal_real("NaN"), None);
        assert_eq!(parse_pascal_real("inf"), None);
    }

    #[test]
    fn parses_pascal_real_with_sign_whitespace_and_exponent() {
        assert_eq!(parse_pascal_real("  +0.5 "), Some(0.5));
        assert_eq!(parse_pascal_real("-0.25E+2"), Some(-25.0));
        assert_eq!(parse_pascal_real("1_024.0e-2"), Some(10.24));
    }

    #[test]
    fn rejects_pascal_real_overflow_and_malformed_text() {
        assert_eq!(parse_pascal_real("1.0e999"), None);
        assert_eq!(parse_pascal_real("1.0e"), None);
        assert_eq!(parse_pascal_real("1._0"), None);
        assert_eq!(parse_pascal_real("1__0.0"), None);
    }

    #[test]
    fn matches_pascal_numeric_text() {
        assert!(is_pascal_numeric("42"));
        assert!(is_pascal_numeric("-7"));
        assert!(is_pascal_numeric("3.0E-4"));
        assert!(is_pascal_numeric("1_000"));
        assert!(!is_pascal_numeric("5."));
        assert!(!is_pascal_numeric("NaN"));
    }

    #[test]
    fn rejects_non_pascal_numeric_variants() {
        assert!(is_pascal_numeric(" +1_024 "));
        assert!(!is_pascal_numeric("1e3"));
        assert!(!is_pascal_numeric("_1"));
        assert!(!is_pascal_numeric("1__0"));
        assert!(!is_pascal_numeric("1.0e"));
    }
}
