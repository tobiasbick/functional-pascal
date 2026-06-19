use crate::{Console, DamageRegion, ViewRect};

/// Host-managed widget that fills a view rectangle with one CRT color.
///
/// Spec: `docs/pascal/std/tui/app/README.md`
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
        DamageRegion::Rect(dirty) => rect.intersection(dirty),
    }
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
