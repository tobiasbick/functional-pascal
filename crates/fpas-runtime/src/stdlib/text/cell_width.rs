//! Shared Unicode display-width policy for terminal cell layout.
//!
//! **Documentation:** `docs/pascal/std/console/cells-frames.md`

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Return the number of terminal columns occupied by `ch`.
///
/// Ambiguous-width characters use neutral width. Combining marks and other zero-width
/// characters return `0` and do not advance layout on their own.
#[must_use]
pub fn display_width(ch: char) -> u8 {
    match UnicodeWidthChar::width(ch) {
        Some(0) => 0,
        Some(1) => 1,
        Some(width) => width.min(2) as u8,
        None => 0,
    }
}

/// Returns the terminal width of one renderable extended grapheme cluster.
///
/// Empty text, multiple clusters, and zero-width clusters return `None` so cell-oriented
/// renderers can reject values that cannot occupy one logical cell.
#[must_use]
pub fn grapheme_cell_width(text: &str) -> Option<u8> {
    let bytes = text.as_bytes();
    // Single printable ASCII byte is always one terminal column.
    if bytes.len() == 1 && (0x20..=0x7E).contains(&bytes[0]) {
        return Some(1);
    }
    let mut graphemes = text.graphemes(true);
    let grapheme = graphemes.next()?;
    if graphemes.next().is_some() {
        return None;
    }
    let width = UnicodeWidthStr::width(grapheme).min(2) as u8;
    (width > 0).then_some(width)
}

/// Split text into its extended grapheme clusters.
#[must_use]
pub fn split_graphemes(text: &str) -> Vec<String> {
    if text.is_ascii() {
        let mut out = Vec::with_capacity(text.len());
        for index in 0..text.len() {
            out.push(text[index..index + 1].to_owned());
        }
        return out;
    }
    text.graphemes(true).map(str::to_owned).collect()
}

/// Sum display widths for every extended grapheme cluster in `text`.
///
/// Measuring clusters keeps joined emoji and a base glyph with combining marks together as one
/// renderable unit. A cluster never contributes more than two terminal columns.
#[must_use]
pub fn str_display_width(text: &str) -> i64 {
    // Printable ASCII is one column per byte; controls and non-ASCII use the Unicode path.
    if text.bytes().all(|byte| (0x20..=0x7E).contains(&byte)) {
        return text.len() as i64;
    }
    text.graphemes(true)
        .map(|grapheme| UnicodeWidthStr::width(grapheme).min(2) as i64)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one_column_per_scalar() {
        assert_eq!(display_width('A'), 1);
        assert_eq!(str_display_width("Hello"), 5);
    }

    #[test]
    fn box_drawing_is_one_column() {
        assert_eq!(display_width('═'), 1);
        assert_eq!(display_width('╔'), 1);
    }

    #[test]
    fn wide_characters_use_two_columns() {
        assert_eq!(display_width('中'), 2);
        assert_eq!(display_width('日'), 2);
        assert_eq!(str_display_width("日本"), 4);
    }

    #[test]
    fn combining_marks_do_not_advance() {
        assert_eq!(display_width('\u{0301}'), 0);
        assert_eq!(str_display_width("e\u{0301}"), 1);
    }

    #[test]
    fn joined_emoji_uses_one_grapheme_width() {
        assert_eq!(str_display_width("👩‍💻"), 2);
        assert_eq!(str_display_width("A👩‍💻B"), 4);
    }

    #[test]
    fn cell_width_requires_exactly_one_renderable_grapheme() {
        assert_eq!(grapheme_cell_width("e\u{0301}"), Some(1));
        assert_eq!(grapheme_cell_width("👩‍💻"), Some(2));
        assert_eq!(grapheme_cell_width("AB"), None);
        assert_eq!(grapheme_cell_width("\u{0301}"), None);
    }

    #[test]
    fn split_preserves_combined_and_joined_graphemes() {
        assert_eq!(split_graphemes("Ae\u{0301}👩‍💻"), ["A", "e\u{0301}", "👩‍💻"]);
    }
}
