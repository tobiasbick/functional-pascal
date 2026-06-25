//! Basic dialog controls for the retained TUI widget toolkit.
//!
//! Public Pascal bindings are documented in `docs/pascal/std/tui/app/controls.md`.

mod button;
mod checkbox;
mod input_line;
mod label;
mod list_box;
mod memo;
mod radio;
mod scroll_bar;
mod scroll_view;

#[cfg(test)]
mod tests;

pub use button::{ButtonStyle, ButtonWidget};
pub use checkbox::{CheckBoxStyle, CheckBoxWidget};
pub use input_line::{InputLineStyle, InputLineWidget};
pub use label::{LabelStyle, LabelWidget};
pub use list_box::{ListBoxItem, ListBoxStyle, ListBoxWidget};
pub use memo::{MemoStyle, MemoWidget};
pub use radio::{RadioGroupStyle, RadioGroupWidget, RadioOption};
pub use scroll_bar::{ScrollBarStyle, ScrollBarWidget};
pub use scroll_view::{ScrollViewStyle, ScrollViewWidget};

use crate::text::text_cells_for_paint;
use crate::{Console, ViewRect};

fn paint_chars(
    console: &mut Console,
    rect: ViewRect,
    clip: ViewRect,
    chars: impl Iterator<Item = (usize, char)>,
    color_for_index: impl Fn(usize) -> u8,
    bg: u8,
) {
    if !clip.contains_point(rect.x, rect.y) && rect.y < clip.y {
        return;
    }
    if rect.y < clip.y || rect.y >= clip.y.saturating_add(clip.height) {
        return;
    }

    for (index, ch) in chars {
        let x = rect.x.saturating_add(index as i64);
        if x < rect.x.saturating_add(rect.width)
            && x >= clip.x
            && x < clip.x.saturating_add(clip.width)
        {
            console.write_char_at_crt(x, rect.y, ch, color_for_index(index), bg);
        }
    }
}

fn truncated_chars(text: &str, width: i64) -> impl Iterator<Item = (usize, char)> + Clone + '_ {
    text_cells_for_paint(text, width)
}

fn accelerator_index(text: &str, accelerator: Option<char>) -> Option<usize> {
    let accelerator = accelerator?.to_ascii_lowercase();
    if !accelerator.is_ascii_alphabetic() {
        return None;
    }
    text.chars()
        .position(|ch| ch.to_ascii_lowercase() == accelerator)
}
