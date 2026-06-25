//! Double-line frame chrome painting.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::{Console, DamageRegion, ViewRect};

use super::{FrameGeometry, FrameStyle};
use crate::text::truncate_for_title_slot;

pub(super) fn paint_underlay(
    console: &mut Console,
    geometry: FrameGeometry,
    damage: DamageRegion,
    style: FrameStyle,
) {
    let Some(clip) = damage.clip_rect(geometry.client) else {
        return;
    };
    console.fill_rect_crt(clip, style.client_fg, style.client_bg, ' ');
}

pub(super) fn paint_overlay(
    console: &mut Console,
    geometry: FrameGeometry,
    damage: DamageRegion,
    title: &str,
    style: FrameStyle,
    active: bool,
) {
    let Some(clip) = damage.clip_rect(geometry.outer) else {
        return;
    };
    let (fg, bg) = if active {
        (style.active_fg, style.active_bg)
    } else {
        (style.inactive_fg, style.inactive_bg)
    };
    let outer = geometry.outer;
    let right = outer.x + outer.width - 1;
    let bottom = outer.y + outer.height - 1;

    for x in outer.x..=right {
        paint_cell(console, clip, x, outer.y, '═', fg, bg);
        paint_cell(console, clip, x, bottom, '═', fg, bg);
    }
    for y in outer.y..=bottom {
        paint_cell(console, clip, outer.x, y, '║', fg, bg);
        paint_cell(console, clip, right, y, '║', fg, bg);
    }
    paint_cell(console, clip, outer.x, outer.y, '╔', fg, bg);
    paint_cell(console, clip, right, outer.y, '╗', fg, bg);
    paint_cell(console, clip, outer.x, bottom, '╚', fg, bg);
    paint_cell(console, clip, right, bottom, '╝', fg, bg);

    if let Some(slot) = geometry.buttons.close {
        paint_cell(console, clip, slot.x, slot.y, '■', fg, bg);
    }
    if let Some(slot) = geometry.buttons.zoom {
        paint_cell(console, clip, slot.x, slot.y, '▲', fg, bg);
    }
    if let Some(slot) = geometry.buttons.zoom_back {
        paint_cell(console, clip, slot.x, slot.y, '▼', fg, bg);
    }
    if let Some(slot) = geometry.buttons.title {
        paint_title(console, clip, slot, title, fg, bg);
    }
}

fn paint_title(console: &mut Console, clip: ViewRect, slot: ViewRect, title: &str, fg: u8, bg: u8) {
    let width = slot.width.max(0) as usize;
    if width == 0 {
        return;
    }
    for (offset, ch) in truncate_for_title_slot(title, width) {
        paint_cell(console, clip, slot.x + offset as i64, slot.y, ch, fg, bg);
    }
}

fn paint_cell(console: &mut Console, clip: ViewRect, x: i64, y: i64, ch: char, fg: u8, bg: u8) {
    if clip.contains_point(x, y) {
        console.write_char_at_crt(x, y, ch, fg, bg);
    }
}
