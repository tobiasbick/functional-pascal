//! A single bounded grammar for stored fields and literal array positions.

pub(super) enum Part<'a> {
    Field(&'a str),
    Index(u64),
}

pub(super) fn qualified_root(name: &str) -> bool {
    name.split('.')
        .all(|part| !part.is_empty() && identifier_length(part) == part.len())
}

pub(super) fn local(path: &str) -> Result<(&str, Vec<Part<'_>>), String> {
    let length = identifier_length(path);
    if length == 0 {
        return Err("Trace roots must be identifiers.".into());
    }
    Ok((&path[..length], suffix(&path[length..])?))
}

pub(super) fn suffix(mut text: &str) -> Result<Vec<Part<'_>>, String> {
    let mut parts = Vec::new();
    while !text.is_empty() {
        if parts.len() == 16 {
            return Err("Trace paths allow at most 16 field/index steps.".into());
        }
        if let Some(tail) = text.strip_prefix('.') {
            let length = identifier_length(tail);
            if length == 0 {
                return Err("A trace field requires an identifier.".into());
            }
            parts.push(Part::Field(&tail[..length]));
            text = &tail[length..];
        } else if let Some(tail) = text.strip_prefix('[') {
            let end = tail.find(']').ok_or("Unclosed trace array index")?;
            let literal = &tail[..end];
            if literal.is_empty()
                || !literal.bytes().all(|b| b.is_ascii_digit())
                || (literal.len() > 1 && literal.starts_with('0'))
            {
                return Err(
                    "Trace indexes require canonical non-negative decimal integer literals.".into(),
                );
            }
            let index = literal
                .parse::<u64>()
                .ok()
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or("Trace index exceeds integer range")?;
            parts.push(Part::Index(index));
            text = &tail[end + 1..];
        } else {
            return Err("Trace paths allow only stored fields and literal array indexes.".into());
        }
    }
    Ok(parts)
}

fn identifier_length(text: &str) -> usize {
    if !text
        .as_bytes()
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
    {
        return 0;
    }
    text.bytes()
        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
        .count()
}
