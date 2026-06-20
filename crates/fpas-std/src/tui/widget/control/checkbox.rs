use crate::{CommandId, Console, DamageRegion, ViewRect};

use super::{accelerator_index, clip_rect_to_damage, paint_chars, truncated_chars};

/// CRT colors used while painting a checkbox control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckBoxStyle {
    /// Normal checkbox background.
    pub bg: u8,
    /// Normal checkbox foreground.
    pub fg: u8,
    /// Background used while focused.
    pub active_bg: u8,
    /// Foreground used while focused.
    pub active_fg: u8,
    /// Accelerator letter foreground.
    pub accelerator_fg: u8,
    /// Foreground used while disabled.
    pub disabled_fg: u8,
}

impl Default for CheckBoxStyle {
    fn default() -> Self {
        Self {
            bg: 7,
            fg: 0,
            active_bg: 0,
            active_fg: 15,
            accelerator_fg: 4,
            disabled_fg: 8,
        }
    }
}

/// Focusable checkbox for modal dialogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckBoxWidget {
    /// Visible label after the checkbox mark.
    pub label: String,
    /// Optional ASCII accelerator letter to highlight.
    pub accelerator: Option<char>,
    /// Optional command emitted by the host when this checkbox is toggled.
    pub command_id: Option<CommandId>,
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Whether the checkbox currently has focus.
    pub focused: bool,
    /// Whether the checkbox accepts activation.
    pub enabled: bool,
    /// CRT style used for painting.
    pub style: CheckBoxStyle,
}

impl CheckBoxWidget {
    /// Create an enabled checkbox.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        accelerator: Option<char>,
        command_id: Option<CommandId>,
        style: CheckBoxStyle,
    ) -> Self {
        Self {
            label: label.into(),
            accelerator,
            command_id,
            checked: false,
            focused: false,
            enabled: true,
            style,
        }
    }

    /// Toggle the checked state when enabled.
    ///
    /// Returns `true` when the checked state changed.
    pub fn toggle(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        self.checked = !self.checked;
        true
    }

    /// Paint the checkbox into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        let (fg, bg) = self.colors();
        console.fill_rect_crt(clip, fg, bg, ' ');

        let text = self.display_text();
        let label_start = 4;
        let accelerator =
            accelerator_index(&self.label, self.accelerator).map(|index| index + label_start);
        paint_chars(
            console,
            rect,
            clip,
            truncated_chars(&text, rect.width),
            |index| {
                if !self.enabled {
                    self.style.disabled_fg
                } else if Some(index) == accelerator && !self.focused {
                    self.style.accelerator_fg
                } else {
                    fg
                }
            },
            bg,
        );
    }

    fn display_text(&self) -> String {
        let mark = if self.checked { 'x' } else { ' ' };
        format!("[{mark}] {}", self.label)
    }

    fn colors(&self) -> (u8, u8) {
        if !self.enabled {
            (self.style.disabled_fg, self.style.bg)
        } else if self.focused {
            (self.style.active_fg, self.style.active_bg)
        } else {
            (self.style.fg, self.style.bg)
        }
    }
}
