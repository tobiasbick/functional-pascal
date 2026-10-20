//! Bounded numeric format specifications for string.Format.

use std::iter::Peekable;
use std::str::Chars;

pub(super) const MAX_WIDTH: usize = 1_000_000;
pub(super) const MAX_PRECISION: usize = 1024;
pub(super) const MAX_NEW_FIELD_BYTES: usize = 1_000_000;

pub(super) struct Spec {
    pub kind: char,
    pub width: usize,
    pub precision: Option<usize>,
    pub left: bool,
    pub zero: bool,
}

impl Spec {
    pub fn parse(chars: &mut Peekable<Chars<'_>>) -> Result<Self, String> {
        let mut left = false;
        let mut zero = false;
        loop {
            match chars.peek() {
                Some('-') if !left => left = true,
                Some('0') if !zero => zero = true,
                Some('-' | '0') => return Err("duplicate format flag".into()),
                _ => break,
            }
            chars.next();
        }
        let width = digits(chars, MAX_WIDTH)?.unwrap_or(0);
        let precision = if chars.peek() == Some(&'.') {
            chars.next();
            Some(digits(chars, MAX_PRECISION)?.ok_or("precision must contain decimal digits")?)
        } else {
            None
        };
        let kind = chars.next().ok_or("incomplete format specifier")?;
        if !matches!(kind, 'd' | 'f' | 'e' | 'E' | 's') {
            return Err(format!("unknown specifier `%{kind}`"));
        }
        if kind == 'd' && precision.is_some() {
            return Err("precision is supported only for `%f`, `%e` and `%E`".into());
        }
        if kind == 's' && (left || zero || width != 0 || precision.is_some()) {
            return Err("format flags, width and precision apply only to numbers".into());
        }
        Ok(Self {
            kind,
            width,
            precision,
            left,
            zero,
        })
    }

    pub fn extended(&self) -> bool {
        self.left
            || self.zero
            || self.width != 0
            || self.precision.is_some()
            || matches!(self.kind, 'e' | 'E')
    }
}

fn digits(chars: &mut Peekable<Chars<'_>>, limit: usize) -> Result<Option<usize>, String> {
    let mut number = None;
    while let Some(character @ '0'..='9') = chars.peek().copied() {
        chars.next();
        let value = number
            .unwrap_or(0_usize)
            .checked_mul(10)
            .and_then(|value| value.checked_add((character as u8 - b'0') as usize))
            .filter(|value| *value <= limit)
            .ok_or_else(|| format!("format field exceeds maximum {limit}"))?;
        number = Some(value);
    }
    Ok(number)
}
