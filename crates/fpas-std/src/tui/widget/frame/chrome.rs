//! Double-line frame chrome painting.
//!
//! **Documentation:** `docs/pascal/std/tui/app/frames.md`

use crate::{Console, DamageRegion, ViewRect};

use super::{FrameGeometry, FrameStyle};

pub(super) fn paint_underlay(
    console: &mut Console,
    geometry: FrameGeometry,
    damage: DamageRegion,
    style: FrameStyle,
) {
    let Some(clip) = clip_to_damage(geometry.client, damage) else {
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
    let Some(clip) = clip_to_damage(geometry.outer, damage) else {
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
    let mut chars = title.chars();
    for index in 0..width {
        let Some(mut ch) = chars.next() else {
            break;
        };
        if index + 1 == width && chars.next().is_some() {
            ch = '…';
        }
        paint_cell(console, clip, slot.x + index as i64, slot.y, ch, fg, bg);
    }
}

fn paint_cell(console: &mut Console, clip: ViewRect, x: i64, y: i64, ch: char, fg: u8, bg: u8) {
    if clip.contains_point(x, y) {
        console.write_char_at_crt(x, y, ch, fg, bg);
    }
}

fn clip_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => rect.intersection(dirty),
    }
}
