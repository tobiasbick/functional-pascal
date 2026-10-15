//! Terminal-safe rendering for structured diagnostics.

use crate::Diagnostic;

mod display;
mod excerpt;
mod json;
pub use json::{render_json, render_program_stderr_json};

/// Explicit presentation policy chosen by the owner of the final output stream.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextStyle {
    /// Terminal-safe text without ANSI styling.
    #[default]
    Plain,
    /// Red errors and yellow warnings, with resets around every styled segment.
    Color,
}

impl TextStyle {
    fn push(self, output: &mut String, text: &str, warning: bool) {
        if self == Self::Color {
            output.push_str(if warning { "\x1b[33m" } else { "\x1b[31m" });
        }
        output.push_str(text);
        if self == Self::Color {
            output.push_str("\x1b[0m");
        }
    }
}

/// Renders a diagnostic with a source path.
///
/// Printable Unicode is preserved. Control characters in paths and line content are escaped,
/// while `CRLF`, bare `CR`, and `LF` in messages or help text become consistently prefixed lines.
#[must_use]
pub fn render(path: &str, diagnostic: &Diagnostic) -> String {
    render_text(Some(path), None, diagnostic, TextStyle::Plain)
}

/// Renders a diagnostic when no source path is available.
///
/// Printable Unicode is preserved. Control characters in line content are escaped, while
/// `CRLF`, bare `CR`, and `LF` become consistently prefixed lines.
#[must_use]
pub fn render_without_path(diagnostic: &Diagnostic) -> String {
    render_text(None, None, diagnostic, TextStyle::Plain)
}

/// Renders a terminal-safe diagnostic, optionally showing its matching source text.
///
/// The caller owns source identity and output policy. No filesystem or environment
/// access occurs here. Invalid source ranges fall back to the compact header/help.
#[must_use]
pub fn render_text(
    path: Option<&str>,
    source: Option<&str>,
    diagnostic: &Diagnostic,
    style: TextStyle,
) -> String {
    let mut rendered = String::new();
    if let Some(path) = path {
        push_escaped(&mut rendered, path);
        rendered.push(':');
    }
    if let Some(span) = diagnostic.span {
        rendered.push_str(&span.line().to_string());
        rendered.push(':');
        rendered.push_str(&span.column().to_string());
        rendered.push_str(": ");
    } else if path.is_some() {
        rendered.push(' ');
    }
    let severity = if diagnostic.is_warning() {
        "warning"
    } else {
        "error"
    };
    style.push(
        &mut rendered,
        &format!("{severity}[{}]", diagnostic.code),
        diagnostic.is_warning(),
    );
    rendered.push_str(": ");
    push_lines(&mut rendered, &diagnostic.message, "", "\n  message: ");

    if let Some(source) = source {
        excerpt::push(&mut rendered, source, diagnostic, style);
    }

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
