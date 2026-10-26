//! Shared Markdown presentation for hover and completion documentation.

/// Use the registered grammar and wrap explicit parameters without splitting nested types.
pub(crate) fn signature_markdown(label: &str) -> String {
    let signature = multiline_signature(label);
    let longest_ticks = label.split(|ch| ch != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(3.max(longest_ticks + 1));
    format!("{fence}fpas\n{signature}\n{fence}")
}

pub(crate) fn declaration_markdown(label: &str, documentation: Option<&str>) -> String {
    let mut markdown = signature_markdown(label);
    if let Some(documentation) = documentation.filter(|text| !text.trim().is_empty()) {
        markdown.push_str("\n\n---\n\n");
        markdown.push_str(documentation);
    }
    markdown
}

fn multiline_signature(label: &str) -> String {
    let Some(open) = label.find('(') else {
        return label.to_owned();
    };
    let mut depth = 0;
    let mut quoted = false;
    let mut separators = Vec::new();
    let mut chars = label
        .char_indices()
        .skip_while(|(offset, _)| *offset < open)
        .peekable();
    while let Some((offset, ch)) = chars.next() {
        if ch == '\'' {
            if quoted && chars.peek().is_some_and(|(_, next)| *next == '\'') {
                chars.next();
            } else {
                quoted = !quoted;
            }
        }
        if quoted {
            continue;
        }
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    if separators.is_empty() {
                        break;
                    }
                    let mut result = format!("{}\n", &label[..=open]);
                    let mut start = open + 1;
                    for end in separators {
                        result.push_str("  ");
                        result.push_str(label[start..end].trim());
                        result.push_str(";\n");
                        start = end + 1;
                    }
                    result.push_str("  ");
                    result.push_str(label[start..offset].trim());
                    result.push('\n');
                    result.push_str(&label[offset..]);
                    return result;
                }
            }
            ';' if depth == 1 => separators.push(offset),
            _ => {}
        }
    }
    label.to_owned()
}
