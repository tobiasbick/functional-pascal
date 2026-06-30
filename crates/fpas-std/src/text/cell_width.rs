//! Unicode display-width policy for terminal cell layout.
//!
//! **Documentation:** `docs/pascal/std/tui/cell-width.md`

use unicode_width::UnicodeWidthChar;

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

/// Sum display widths for every scalar in `text`.
#[must_use]
pub fn str_display_width(text: &str) -> i64 {
    text.chars().map(|ch| i64::from(display_width(ch))).sum()
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

/// Lay out title text into a fixed-width title slot.
#[must_use]
pub fn truncate_for_title_slot(text: &str, max_cols: usize) -> Vec<(usize, char)> {
    layout_display_cells(text, max_cols)
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
    fn title_slot_matches_frame_truncation() {
        assert_eq!(
            truncate_for_title_slot("Long dialog title", 4),
            vec![(0, 'L'), (1, 'o'), (2, 'n'), (3, '…')]
        );
    }

    #[test]
    fn char_display_offset_tracks_wide_chars() {
        assert_eq!(char_display_offset("A日本", 2), 3);
    }
}
