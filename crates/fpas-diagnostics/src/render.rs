//! Terminal-safe rendering for structured diagnostics.

use crate::Diagnostic;

/// Renders a diagnostic with a source path.
///
/// Printable Unicode is preserved. Control characters in paths and line content are escaped,
/// while `CRLF`, bare `CR`, and `LF` in messages or help text become consistently prefixed lines.
#[must_use]
pub fn render(path: &str, diagnostic: &Diagnostic) -> String {
    render_at(Some(path), diagnostic)
}

/// Renders a diagnostic when no source path is available.
///
/// Printable Unicode is preserved. Control characters in line content are escaped, while
/// `CRLF`, bare `CR`, and `LF` become consistently prefixed lines.
#[must_use]
pub fn render_without_path(diagnostic: &Diagnostic) -> String {
    render_at(None, diagnostic)
}

fn render_at(path: Option<&str>, diagnostic: &Diagnostic) -> String {
    let mut rendered = String::new();
    if let Some(path) = path {
        push_escaped(&mut rendered, path);
        rendered.push(':');
    }
    rendered.push_str(&diagnostic.span.line().to_string());
    rendered.push(':');
    rendered.push_str(&diagnostic.span.column().to_string());
    rendered.push_str(": ");
    rendered.push_str(if diagnostic.is_warning() {
        "warning"
    } else {
        "error"
    });
    rendered.push('[');
    rendered.push_str(&diagnostic.code.to_string());
    rendered.push_str("]: ");
    push_lines(&mut rendered, &diagnostic.message, "", "\n  message: ");

    if let Some(help) = diagnostic
        .help
        .as_deref()
        .filter(|help| !help.trim().is_empty())
    {
        push_lines(&mut rendered, help, "\n  help: ", "\n  help: ");
    }

    rendered
}

fn push_lines(output: &mut String, text: &str, first_prefix: &str, next_prefix: &str) {
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut index = 0;
    let mut prefix = first_prefix;

    loop {
        if index == bytes.len() {
            output.push_str(prefix);
            push_escaped(output, &text[start..index]);
            return;
        }

        let newline_len = match bytes[index] {
            b'\n' => 1,
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => 2,
            b'\r' => 1,
            _ => {
                index += 1;
                continue;
            }
        };

        output.push_str(prefix);
        push_escaped(output, &text[start..index]);
        prefix = next_prefix;
        index += newline_len;
        start = index;
    }
}

fn push_escaped(output: &mut String, text: &str) {
    for character in text.chars() {
        if character.is_control() {
            output.extend(character.escape_default());
        } else {
            output.push(character);
        }
    }
}
