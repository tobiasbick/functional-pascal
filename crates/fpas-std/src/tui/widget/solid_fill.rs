use crate::{Console, DamageRegion, ViewRect};

/// Host-managed widget that fills a view rectangle with one CRT color.
///
/// Spec: `docs/pascal/std/tui/app.md`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolidFillWidget {
    /// Packed CRT background color (`0..=15`).
    pub fill_color: u8,
    /// Optional packed CRT foreground color for [`Self::fill_char`].
    pub text_color: Option<u8>,
    /// Optional tile character. When absent, the view is filled with spaces.
    pub fill_char: Option<char>,
}

impl SolidFillWidget {
    /// Resolve the foreground color used while painting.
    #[must_use]
    pub fn foreground_color(self) -> u8 {
        match self.fill_char {
            None | Some(' ') => self.fill_color,
            Some(_) => self.text_color.unwrap_or(7),
        }
    }

    /// Resolve the character written into every cell of the view.
    #[must_use]
    pub fn character(self) -> char {
        self.fill_char.unwrap_or(' ')
    }

    /// Paint the widget into `rect`, clipped to `damage`.
    pub fn paint(self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };

        console.fill_rect_crt(
            clip,
            self.foreground_color(),
            self.fill_color,
            self.character(),
        );
    }
}

fn clip_rect_to_damage(rect: ViewRect, damage: DamageRegion) -> Option<ViewRect> {
    match damage {
        DamageRegion::FullFrame => Some(rect),
        DamageRegion::Rect(dirty) => intersect_view_rects(rect, dirty),
    }
}

fn intersect_view_rects(left: ViewRect, right: ViewRect) -> Option<ViewRect> {
    let left_right = left.x.saturating_add(left.width.max(0));
    let left_bottom = left.y.saturating_add(left.height.max(0));
    let right_right = right.x.saturating_add(right.width.max(0));
    let right_bottom = right.y.saturating_add(right.height.max(0));

    let x = left.x.max(right.x);
    let y = left.y.max(right.y);
    let right = left_right.min(right_right);
    let bottom = left_bottom.min(right_bottom);

    if right <= x || bottom <= y {
        return None;
    }

    Some(ViewRect {
        x,
        y,
        width: right - x,
        height: bottom - y,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use fpas_bytecode::SourceLocation;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn solid_fill_paints_blue_background() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        SolidFillWidget {
            fill_color: 1,
            text_color: None,
            fill_char: None,
        }
        .paint(
            &mut console,
            ViewRect {
                x: 0,
                y: 0,
                width: 5,
                height: 2,
            },
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(test_location()).unwrap();
        assert_eq!(console.test_cell(1, 1), (' ', 1, 1));
        assert_eq!(console.test_cell(5, 2), (' ', 1, 1));
    }

    #[test]
    fn solid_fill_tiles_character_with_text_color() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        SolidFillWidget {
            fill_color: 1,
            text_color: Some(14),
            fill_char: Some('.'),
        }
        .paint(
            &mut console,
            ViewRect {
                x: 1,
                y: 1,
                width: 2,
                height: 1,
            },
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(test_location()).unwrap();
        assert_eq!(console.test_cell(2, 2), ('.', 14, 1));
        assert_eq!(console.test_cell(3, 2), ('.', 14, 1));
    }
}
