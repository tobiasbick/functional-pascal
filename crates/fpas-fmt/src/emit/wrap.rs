//! Line-width measurement and wrapping helpers.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#line-width-v2`

use crate::style::{INDENT_WIDTH, MAX_LINE_WIDTH};
use unicode_width::UnicodeWidthStr;

use super::Emitter;

/// Visible width of `text` in terminal-style Unicode display columns.
pub(crate) fn text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Renders `emit` into a fresh buffer and returns the text.
pub(crate) fn measure_emit(emit: impl FnOnce(&mut Emitter)) -> String {
    let mut emitter = Emitter::new();
    emit(&mut emitter);
    emitter.finish()
}

/// Returns `true` when `column + addition` exceeds [`MAX_LINE_WIDTH`].
pub(crate) fn exceeds_width(column: usize, addition: usize) -> bool {
    column + addition > MAX_LINE_WIDTH
}

/// Emits a comma-separated list, wrapping after commas when over max width.
///
/// `first_line_prefix` is written at column 0 on the first line (e.g. `uses `).
/// Continuation lines start at `continuation_spaces` columns from the line start.
/// `terminator` is appended to the last line (e.g. `;`).
pub(crate) fn emit_wrapped_comma_list(
    emitter: &mut Emitter,
    first_line_prefix: &str,
    continuation_spaces: usize,
    items: &[String],
    terminator: &str,
) {
    assert!(!items.is_empty());

    let single_line = format!("{first_line_prefix}{}{terminator}", items.join(", "));
    if text_width(&single_line) <= MAX_LINE_WIDTH {
        emitter.write(&single_line);
        emitter.write("\n");
        return;
    }

    emitter.write(first_line_prefix.trim_end());
    emitter.write("\n");

    let mut line = String::new();
    let cont = " ".repeat(continuation_spaces);

    for (index, item) in items.iter().enumerate() {
        let is_last = index + 1 == items.len();
        let piece = if line.is_empty() {
            format!("{cont}{item}")
        } else {
            format!("{line}, {item}")
        };

        let with_term = if is_last {
            format!("{piece}{terminator}")
        } else {
            piece.clone()
        };

        if text_width(&with_term) <= MAX_LINE_WIDTH {
            line = piece;
            continue;
        }

        if line.is_empty() {
            let mut solo = format!("{cont}{item}");
            if is_last {
                solo.push_str(terminator);
            }
            emitter.write(&solo);
            if !is_last {
                emitter.write(",");
            }
            emitter.write("\n");
            line.clear();
            continue;
        }

        emitter.write(&line);
        emitter.write(",\n");
        line = format!("{cont}{item}");
        if is_last {
            line.push_str(terminator);
            emitter.write(&line);
            emitter.write("\n");
            line.clear();
        }
    }

    if !line.is_empty() {
        emitter.write(&line);
        emitter.write(terminator);
        emitter.write("\n");
    }
}

/// Emits a semicolon-separated list inside parentheses, wrapping after `;` when over max width.
///
/// `open_prefix` ends with `(` (e.g. `function Foo(`). Each item is one formal parameter.
/// `close_suffix` starts after the closing `)` (e.g. `: integer` or empty before `;`).
pub(crate) fn emit_wrapped_semicolon_paren_list(
    emitter: &mut Emitter,
    open_prefix: &str,
    items: &[String],
    close_suffix: &str,
) {
    if items.is_empty() {
        emitter.write(open_prefix);
        emitter.write(")");
        emitter.write(close_suffix);
        return;
    }

    let inline = format!(
        "{open_prefix}{items}){close_suffix}",
        items = items.join("; ")
    );
    if text_width(&inline) <= MAX_LINE_WIDTH {
        emitter.write(&inline);
        return;
    }

    emitter.write(open_prefix);
    emitter.write("\n");

    let continuation_spaces = INDENT_WIDTH;
    let cont = " ".repeat(continuation_spaces);

    for (index, item) in items.iter().enumerate() {
        let is_last = index + 1 == items.len();
        let line = if is_last {
            format!("{cont}{item}\n")
        } else {
            format!("{cont}{item};\n")
        };
        emitter.write(&line);
    }

    emitter.write(")");
    emitter.write(close_suffix);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comma_list_stays_single_line_when_short() {
        let mut emitter = Emitter::new();
        emit_wrapped_comma_list(
            &mut emitter,
            "uses ",
            INDENT_WIDTH,
            &[String::from("Std.Console"), String::from("Std.Conv")],
            ";",
        );
        assert_eq!(emitter.finish(), "uses Std.Console, Std.Conv;\n");
    }

    #[test]
    fn comma_list_wraps_long_items() {
        let mut emitter = Emitter::new();
        emit_wrapped_comma_list(
            &mut emitter,
            "uses ",
            INDENT_WIDTH,
            &["A".repeat(50), "B".repeat(50)],
            ";",
        );
        let out = emitter.finish();
        assert!(out.starts_with("uses\n"));
        assert!(out.contains(','));
        assert!(out.ends_with(";\n"));
    }

    #[test]
    fn text_width_counts_combining_and_wide_unicode_columns() {
        assert_eq!(text_width("e\u{301}"), 1);
        assert_eq!(text_width("界"), 2);
        assert_eq!(text_width(&format!("{}界", "a".repeat(98))), 100);
        assert_eq!(text_width(&format!("{}界", "a".repeat(99))), 101);
    }
}
