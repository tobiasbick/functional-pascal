//! Replacement expansion and splitting never construct an unbounded intermediate output.
use super::{Budget, Failure, MAX_TEXT, check};
use crate::Value;

fn append(output: &mut String, text: &str, budget: &mut Budget) -> Result<(), Failure> {
    budget.text(text)?;
    output.push_str(text);
    Ok(())
}

fn expand(
    output: &mut String,
    mut template: &str,
    captures: &regex::Captures<'_>,
    budget: &mut Budget,
) -> Result<(), Failure> {
    while let Some(dollar) = template.find('$') {
        append(output, &template[..dollar], budget)?;
        template = &template[dollar + 1..];
        if let Some(rest) = template.strip_prefix('$') {
            append(output, "$", budget)?;
            template = rest;
            continue;
        }
        let reference = if let Some(braced) = template.strip_prefix('{') {
            braced.find('}').map(|end| (&braced[..end], end + 2))
        } else {
            let end = template
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                .count();
            (end > 0).then(|| (&template[..end], end))
        };
        let Some((name, consumed)) = reference else {
            append(output, "$", budget)?;
            continue;
        };
        let captured = if name.bytes().all(|b| b.is_ascii_digit()) && !name.is_empty() {
            name.parse::<usize>()
                .ok()
                .and_then(|index| captures.get(index))
        } else {
            captures.name(name)
        };
        if let Some(value) = captured {
            append(output, value.as_str(), budget)?;
        }
        template = &template[consumed..];
    }
    append(output, template, budget)
}

pub(super) fn replace(
    pattern: &regex::Regex,
    text: &str,
    template: &str,
    limit: usize,
) -> Result<Value, Failure> {
    check(template.len(), MAX_TEXT, "ReplacementBytes")?;
    let mut output = String::new();
    let mut budget = Budget::default();
    let mut previous = 0;
    for (index, captures) in pattern.captures_iter(text).enumerate() {
        if limit != 0 && index >= limit {
            break;
        }
        budget.item()?;
        let whole = captures
            .get(0)
            .unwrap_or_else(|| unreachable!("group zero always exists"));
        append(&mut output, &text[previous..whole.start()], &mut budget)?;
        expand(&mut output, template, &captures, &mut budget)?;
        previous = whole.end();
    }
    append(&mut output, &text[previous..], &mut budget)?;
    Ok(Value::Str(output.into()))
}

pub(super) fn split(pattern: &regex::Regex, text: &str, limit: usize) -> Result<Value, Failure> {
    let mut parts = Vec::new();
    let mut budget = Budget::default();
    let iterator = pattern.splitn(text, if limit == 0 { usize::MAX } else { limit });
    for part in iterator {
        budget.item()?;
        budget.text(part)?;
        parts.push(Value::Str(part.into()));
    }
    Ok(Value::Array(parts.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expansion_matches_engine_syntax() {
        let regex = regex::Regex::new(r"(?P<word>\w+)(?:-(\d+))?").unwrap();
        for text in ["ab-42 cd", "é", "", "x-2"] {
            for template in [
                "$0",
                "$1/$2",
                "${word}x",
                "$$",
                "$missing",
                "$",
                "${",
                "${}",
                "$1x",
                "${1}x",
                "$999999999999999999999999",
            ] {
                let Value::Str(actual) = replace(&regex, text, template, 0)
                    .unwrap_or_else(|_| panic!("bounded fixture"))
                else {
                    panic!("text result");
                };
                assert_eq!(
                    actual.as_ref(),
                    regex.replace_all(text, template).as_ref(),
                    "{text:?} {template:?}"
                );
            }
        }
    }
}
