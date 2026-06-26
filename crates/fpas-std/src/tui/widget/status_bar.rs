//! Host-managed status bar painted in Rust from a Pascal segment model.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

use crate::text::{layout_display_cells, str_display_width};
use crate::{Console, DamageRegion, ViewRect};

/// One declarative status segment supplied from Pascal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarSegment {
    /// Visible text (may include leading/trailing spaces for padding).
    pub text: String,
    /// When true, the segment is anchored to the right edge of the bar.
    pub align_right: bool,
}

/// CRT colors used while painting a status bar widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusBarStyle {
    /// Bar background.
    pub bar_bg: u8,
    /// Bar foreground used for segment text.
    pub bar_fg: u8,
}

impl Default for StatusBarStyle {
    fn default() -> Self {
        Self {
            bar_bg: 7,
            bar_fg: 0,
        }
    }
}

/// Host-managed status bar widget state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarWidget {
    pub segments: Vec<StatusBarSegment>,
    pub style: StatusBarStyle,
}

impl StatusBarWidget {
    /// Creates a status bar widget from Pascal-supplied model data.
    #[must_use]
    pub fn new(segments: Vec<StatusBarSegment>, style: StatusBarStyle) -> Self {
        Self { segments, style }
    }

    /// Replaces the segment model at runtime.
    pub fn set_segments(&mut self, segments: Vec<StatusBarSegment>) {
        self.segments = segments;
    }

    /// Paint the status bar clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = damage.clip_rect(rect) else {
            return;
        };

        console.fill_rect_crt(clip, self.style.bar_fg, self.style.bar_bg, ' ');

        let mut left_x = rect.x;
        for segment in self.segments.iter().filter(|segment| !segment.align_right) {
            left_x = paint_segment(
                console,
                rect.y,
                left_x,
                rect.x + rect.width,
                segment,
                self.style,
            );
        }

        let mut right_x = rect.x + rect.width;
        for segment in self
            .segments
            .iter()
            .rev()
            .filter(|segment| segment.align_right)
        {
            let width = str_display_width(&segment.text);
            if width <= 0 {
                continue;
            }
            right_x = right_x.saturating_sub(width);
            if right_x < rect.x {
                break;
            }
            console.write_text_at_crt(
                right_x,
                rect.y,
                &segment.text,
                self.style.bar_fg,
                self.style.bar_bg,
            );
        }

        let _ = left_x;
    }
}

fn paint_segment(
    console: &mut Console,
    y: i64,
    x: i64,
    max_x: i64,
    segment: &StatusBarSegment,
    style: StatusBarStyle,
) -> i64 {
    let width = str_display_width(&segment.text);
    if width <= 0 || x >= max_x {
        return x;
    }
    let end = x.saturating_add(width).min(max_x);
    let visible = end - x;
    if visible <= 0 {
        return x;
    }
    for (offset, ch) in layout_display_cells(&segment.text, visible as usize) {
        console.write_char_at_crt(x + offset as i64, y, ch, style.bar_fg, style.bar_bg);
    }
    x + visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::Console;
    use fpas_bytecode::SourceLocation;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn status_bar_paints_left_and_right_segments() {
        let mut console = Console::new();
        console.assign_crt().unwrap();
        console.begin_tui_paint(DamageRegion::FullFrame);

        StatusBarWidget::new(
            vec![
                StatusBarSegment {
                    text: " F10 Menu ".into(),
                    align_right: false,
                },
                StatusBarSegment {
                    text: "Ln 1, Col 1".into(),
                    align_right: true,
                },
            ],
            StatusBarStyle::default(),
        )
        .paint(
            &mut console,
            ViewRect {
                x: 0,
                y: 0,
                width: 30,
                height: 1,
            },
            DamageRegion::FullFrame,
        );

        console.finish_tui_paint(loc()).unwrap();
        assert_eq!(console.test_cell(2, 1), ('F', 0, 7));
        assert_eq!(console.test_cell(30, 1), ('1', 0, 7));
    }
}
