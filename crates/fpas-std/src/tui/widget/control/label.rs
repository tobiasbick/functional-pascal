use crate::{Console, DamageRegion, ViewRect};

use super::{clip_rect_to_damage, paint_chars, truncated_chars};

/// CRT colors used while painting a static dialog label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelStyle {
    /// Label background.
    pub bg: u8,
    /// Normal label foreground.
    pub fg: u8,
    /// Accelerator letter foreground.
    pub accelerator_fg: u8,
    /// Foreground used when the label is disabled.
    pub disabled_fg: u8,
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            accelerator_fg: 4,
            disabled_fg: 8,
        }
    }
}

/// Static text label for dialog layouts.
///
/// The optional accelerator marks one ASCII letter for highlighting; command routing is owned by
/// the associated focusable control, not by the label itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelWidget {
    /// Visible label text.
    pub text: String,
    /// Optional ASCII accelerator letter to highlight.
    pub accelerator: Option<char>,
    /// Whether the label paints as enabled.
    pub enabled: bool,
    /// CRT style used for painting.
    pub style: LabelStyle,
}

impl LabelWidget {
    /// Create an enabled label.
    #[must_use]
    pub fn new(text: impl Into<String>, accelerator: Option<char>, style: LabelStyle) -> Self {
        Self {
            text: text.into(),
            accelerator,
            enabled: true,
            style,
        }
    }

    /// Paint the label into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        console.fill_rect_crt(clip, self.foreground_color(), self.style.bg, ' ');

        let accelerator_index = self.accelerator_index();
        paint_chars(
            console,
            rect,
            clip,
            truncated_chars(&self.text, rect.width),
            |index| {
                if !self.enabled {
                    self.style.disabled_fg
                } else if Some(index) == accelerator_index {
                    self.style.accelerator_fg
                } else {
                    self.style.fg
                }
            },
            self.style.bg,
        );
    }

    fn foreground_color(&self) -> u8 {
        if self.enabled {
            self.style.fg
        } else {
            self.style.disabled_fg
        }
    }

    fn accelerator_index(&self) -> Option<usize> {
        let accelerator = self.accelerator?.to_ascii_lowercase();
        if !accelerator.is_ascii_alphabetic() {
            return None;
        }
        self.text
            .chars()
            .position(|ch| ch.to_ascii_lowercase() == accelerator)
    }
}
