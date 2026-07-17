//! Shared Unicode display-width policy for terminal cell layout.
//!
//! **Documentation:** `docs/pascal/std/console/cells-frames.md`

#![allow(dead_code)] // Layout helpers remain reserved for Turbo Vision text layout.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Continuation filler for the second column of a wide character.
pub(crate) const WIDE_CONTINUATION: char = ' ';

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

/// Sum display widths for every extended grapheme cluster in `text`.
///
/// Measuring clusters keeps joined emoji and a base glyph with combining marks together as one
/// renderable unit. A cluster never contributes more than two terminal columns.
#[must_use]
pub fn str_display_width(text: &str) -> i64 {
    text.graphemes(true)
        .map(|grapheme| UnicodeWidthStr::width(grapheme).min(2) as i64)
        .sum()
}

/// Returns the terminal width of one renderable extended grapheme cluster.
///
/// Empty text, multiple clusters, and zero-width clusters return `None` so cell-oriented
/// renderers can reject values that cannot occupy one logical cell.
#[must_use]
pub fn grapheme_cell_width(text: &str) -> Option<u8> {
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
    text.graphemes(true).map(str::to_owned).collect()
}

/// Display-column offset immediately before the scalar at `char_index`.
#[must_use]
#[cfg(test)]
fn char_display_offset(text: &str, char_index: usize) -> usize {
    text.chars()
        .take(char_index)
        .map(|ch| usize::from(display_width(ch)))
        .sum()
}

/// Lay out `(column_offset, char)` pairs for painting up to `max_cols` terminal columns.
#[must_use]
pub fn layout_display_cells(text: &str, max_cols: usize) -> Vec<(usize, char)> {
    if max_cols == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut col = 0usize;
    let mut chars = text.chars().peekable();

    while col < max_cols {
        let Some(ch) = chars.next() else {
            break;
        };
        let width = usize::from(display_width(ch));
        if width == 0 {
            continue;
        }

        let remaining_cols = max_cols.saturating_sub(col);
        let has_following = chars.clone().any(|next| display_width(next) > 0);

        if width > remaining_cols {
            if remaining_cols > 0 {
                result.push((col, '…'));
            }
            break;
        }

        if has_following && col + width >= max_cols {
            result.push((col, '…'));
            break;
        }

        result.push((col, ch));
        col += width;
    }

    result
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
        assert_eq!(char_display_offset("e\u{0301}", 2), 1);
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

    #[test]
    fn layout_truncates_with_ellipsis() {
        assert_eq!(
            layout_display_cells("Long dialog title", 4),
            vec![(0, 'L'), (1, 'o'), (2, 'n'), (3, '…')]
        );
    }

    #[test]
    fn layout_fits_wide_characters() {
        assert_eq!(layout_display_cells("日本", 4), vec![(0, '日'), (2, '本')]);
    }

    #[test]
    fn char_display_offset_tracks_wide_chars() {
        assert_eq!(char_display_offset("A日本", 2), 3);
    }
}
