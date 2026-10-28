//! Decimal presentation keeps integer precision and uses binary64's standard rounding.

pub(super) fn fixed_integer(value: i64, precision: usize) -> String {
    let mut output = value.to_string();
    if precision != 0 {
        output.push('.');
        output.extend(std::iter::repeat_n('0', precision));
    }
    output
}

pub(super) fn scientific_integer(value: i64, precision: usize, upper: bool) -> String {
    let mut digits = value.unsigned_abs().to_string().into_bytes();
    let mut exponent = digits.len() - 1;
    let significant = precision + 1;
    if digits.len() > significant {
        let discarded = &digits[significant..];
        let round_up = discarded[0] > b'5'
            || (discarded[0] == b'5'
                && (discarded[1..].iter().any(|digit| *digit != b'0')
                    || (digits[significant - 1] - b'0') % 2 == 1));
        digits.truncate(significant);
        if round_up {
            let mut carry = true;
            for digit in digits.iter_mut().rev() {
                if *digit == b'9' {
                    *digit = b'0';
                } else {
                    *digit += 1;
                    carry = false;
                    break;
                }
            }
            if carry {
                digits[0] = b'1';
                exponent += 1;
            }
        }
    } else {
        digits.resize(significant, b'0');
    }
    let mut output = String::new();
    if value < 0 {
        output.push('-');
    }
    output.push(digits[0] as char);
    if precision != 0 {
        output.push('.');
        output.extend(digits[1..].iter().map(|digit| *digit as char));
    }
    output.push_str(&format!("{}{exponent:02}", if upper { "E+" } else { "e+" }));
    output
}

pub(super) fn scientific_real(value: f64, precision: usize, upper: bool) -> String {
    let raw = format!("{value:.precision$e}");
    if let Some((mantissa, exponent)) = raw.rsplit_once('e') {
        let (sign, magnitude) = exponent
            .strip_prefix('-')
            .map_or(("+", exponent), |digits| ("-", digits));
        return format!(
            "{mantissa}{}{sign}{magnitude:0>2}",
            if upper { 'E' } else { 'e' }
        );
    }
    raw
}
