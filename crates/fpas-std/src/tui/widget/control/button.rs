use crate::{CommandId, Console, DamageRegion, ViewRect};

use super::{clip_rect_to_damage, paint_chars, truncated_chars};

/// CRT colors used while painting a dialog button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonStyle {
    /// Button background.
    pub bg: u8,
    /// Normal button foreground.
    pub fg: u8,
    /// Background used for focused/default buttons.
    pub active_bg: u8,
    /// Foreground used for focused/default buttons.
    pub active_fg: u8,
    /// Foreground used when the button is disabled.
    pub disabled_fg: u8,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            active_bg: 0,
            active_fg: 15,
            disabled_fg: 8,
        }
    }
}

/// Focusable push button for modal dialogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonWidget {
    /// Visible caption without surrounding button chrome.
    pub caption: String,
    /// Optional command emitted by the host when this button is activated.
    pub command_id: Option<CommandId>,
    /// Whether this button is the dialog's default action.
    pub default: bool,
    /// Whether this button currently has focus.
    pub focused: bool,
    /// Whether this button accepts activation.
    pub enabled: bool,
    /// CRT style used for painting.
    pub style: ButtonStyle,
}

impl ButtonWidget {
    /// Create an enabled button.
    #[must_use]
    pub fn new(
        caption: impl Into<String>,
        command_id: Option<CommandId>,
        style: ButtonStyle,
    ) -> Self {
        Self {
            caption: caption.into(),
            command_id,
            default: false,
            focused: false,
            enabled: true,
            style,
        }
    }

    /// Minimum view width needed to display the button caption and chrome.
    #[must_use]
    pub fn minimum_width(&self) -> i64 {
        self.display_text().chars().count() as i64
    }

    /// Paint the button into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        let (fg, bg) = self.colors();
        console.fill_rect_crt(clip, fg, bg, ' ');

        let text = self.display_text();
        let text_width = text.chars().count() as i64;
        let x = rect.x + (rect.width.saturating_sub(text_width) / 2).max(0);
        let y = rect.y + (rect.height.saturating_sub(1) / 2).max(0);
        let text_rect = ViewRect {
            x,
            y,
            width: rect.width.min(text_width),
            height: 1,
        };
        paint_chars(
            console,
            text_rect,
            clip,
            truncated_chars(&text, text_rect.width),
            |_| fg,
            bg,
        );
    }

    fn colors(&self) -> (u8, u8) {
        if !self.enabled {
            (self.style.disabled_fg, self.style.bg)
        } else if self.focused || self.default {
            (self.style.active_fg, self.style.active_bg)
        } else {
            (self.style.fg, self.style.bg)
        }
    }

    fn display_text(&self) -> String {
        if self.default {
            format!("< {} >", self.caption)
        } else {
            format!("[ {} ]", self.caption)
        }
    }
}
