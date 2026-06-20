//! Basic dialog controls for the retained TUI widget toolkit.
//!
//! These widgets are Rust-internal building blocks for the dialog-controls phase tracked in
//! `docs/future/windows-dialogs/TUI-CODE-REVIEW.md`.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

mod button;
mod label;

#[cfg(test)]
mod tests;

pub use button::{ButtonStyle, ButtonWidget};
pub use label::{LabelStyle, LabelWidget};

use crate::{Console, DamageRegion, ViewRect};

fn clip_rect_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => rect.intersection(dirty),
    }
}

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

fn truncated_chars(text: &str, width: i64) -> impl Iterator<Item = (usize, char)> + '_ {
    let width = width.max(0) as usize;
    text.chars().take(width).enumerate()
}
