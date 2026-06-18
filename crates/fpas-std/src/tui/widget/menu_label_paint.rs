//! Shared label painting helpers for menu bar and popup rows.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::Console;

use super::menu_style::MenuLabelPaint;

/// Paint label text with Turbo Pascal-style shortcut letter highlighting.
pub(in crate::tui::widget) fn paint_labeled_text(
    console: &mut Console,
    x: i64,
    y: i64,
    label: &str,
    shortcut: &str,
    enabled: bool,
    colors: MenuLabelPaint,
) {
    let highlight_index = shortcut_highlight_index(label.trim(), shortcut);
    let mut col = x;
    for (index, ch) in label.chars().enumerate() {
        let cell_fg = if colors.hovered || !enabled {
            colors.fg
        } else if highlight_index == Some(index) {
            colors.accel_fg
        } else {
            colors.fg
        };
        console.write_char_at_crt(col, y, ch, cell_fg, colors.bg);
        col += 1;
    }
}

/// Returns the character index of the shortcut letter inside a padded label.
pub(in crate::tui::widget) fn shortcut_highlight_index(
    label: &str,
    shortcut: &str,
) -> Option<usize> {
    let shortcut = shortcut.chars().next()?;
    if !shortcut.is_ascii_alphabetic() {
        return None;
    }
    let inner = format!(" {label} ");
    inner
        .char_indices()
        .find(|(_, ch)| ch.eq_ignore_ascii_case(&shortcut))
        .map(|(index, _)| index)
}
