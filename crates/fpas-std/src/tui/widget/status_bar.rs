//! Host-managed status bar painted in Rust from a Pascal segment model.
//!
//! Spec: `docs/pascal/std/tui/app/README.md`

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
    pub fn paint(self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
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
            let width = segment.text.chars().count() as i64;
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
    let width = segment.text.chars().count() as i64;
    if width <= 0 || x >= max_x {
        return x;
    }
    let end = x.saturating_add(width).min(max_x);
    let visible = end - x;
    if visible <= 0 {
        return x;
    }
    let text: String = segment.text.chars().take(visible as usize).collect();
    console.write_text_at_crt(x, y, &text, style.bar_fg, style.bar_bg);
    x + visible
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
