use crate::{CommandId, Console, DamageRegion, ViewRect};

use super::{accelerator_index, clip_rect_to_damage, paint_chars, truncated_chars};

/// One option in a retained radio group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioOption {
    /// Visible option label.
    pub label: String,
    /// Optional ASCII accelerator letter to highlight.
    pub accelerator: Option<char>,
    /// Optional command emitted by the host when this option is selected.
    pub command_id: Option<CommandId>,
    /// Whether this option can be focused and selected.
    pub enabled: bool,
}

impl RadioOption {
    /// Create an enabled radio option.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        accelerator: Option<char>,
        command_id: Option<CommandId>,
    ) -> Self {
        Self {
            label: label.into(),
            accelerator,
            command_id,
            enabled: true,
        }
    }
}

/// CRT colors used while painting a radio group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RadioGroupStyle {
    /// Normal radio background.
    pub bg: u8,
    /// Normal radio foreground.
    pub fg: u8,
    /// Background used for the focused option.
    pub active_bg: u8,
    /// Foreground used for the focused option.
    pub active_fg: u8,
    /// Accelerator letter foreground.
    pub accelerator_fg: u8,
    /// Foreground used for disabled options.
    pub disabled_fg: u8,
}

impl Default for RadioGroupStyle {
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

/// Vertical radio-button group for modal dialogs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioGroupWidget {
    /// Options displayed in vertical order.
    pub options: Vec<RadioOption>,
    selected: Option<usize>,
    focused_option: Option<usize>,
    /// Whether the group accepts activation.
    pub enabled: bool,
    /// Whether the group view currently has retained focus.
    pub focused: bool,
    /// CRT style used for painting.
    pub style: RadioGroupStyle,
}

impl RadioGroupWidget {
    /// Create an enabled radio group.
    #[must_use]
    pub fn new(options: Vec<RadioOption>, style: RadioGroupStyle) -> Self {
        let first_enabled = options.iter().position(|option| option.enabled);
        Self {
            options,
            selected: first_enabled,
            focused_option: first_enabled,
            enabled: true,
            focused: false,
            style,
        }
    }

    /// Return the selected option index, if any.
    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Return the focused option index, if any.
    #[must_use]
    pub fn focused_option(&self) -> Option<usize> {
        self.focused_option
    }

    /// Return the command associated with the selected option, if any.
    #[must_use]
    pub fn selected_command(&self) -> Option<CommandId> {
        self.selected
            .and_then(|index| self.options.get(index))
            .and_then(|option| option.command_id)
    }

    /// Select an enabled option by index.
    ///
    /// Returns `true` when selection changed.
    pub fn set_selected(&mut self, index: usize) -> bool {
        if !self.option_is_selectable(index) || self.selected == Some(index) {
            return false;
        }
        self.selected = Some(index);
        self.focused_option = Some(index);
        true
    }

    /// Move focus to an enabled option by index.
    ///
    /// Returns `true` when focus changed.
    pub fn set_focused_option(&mut self, index: usize) -> bool {
        if !self.option_is_selectable(index) || self.focused_option == Some(index) {
            return false;
        }
        self.focused_option = Some(index);
        true
    }

    /// Move focus to the next enabled option, wrapping around.
    pub fn focus_next(&mut self) -> bool {
        self.focus_step(true)
    }

    /// Move focus to the previous enabled option, wrapping around.
    pub fn focus_prev(&mut self) -> bool {
        self.focus_step(false)
    }

    /// Select the currently focused option.
    pub fn select_focused(&mut self) -> bool {
        let Some(index) = self.focused_option else {
            return false;
        };
        self.set_selected(index)
    }

    /// Paint the radio group into `rect`, clipped to `damage`.
    pub fn paint(&self, console: &mut Console, rect: ViewRect, damage: DamageRegion) {
        let Some(clip) = clip_rect_to_damage(rect, damage) else {
            return;
        };
        console.fill_rect_crt(clip, self.style.fg, self.style.bg, ' ');

        let visible_rows = rect.height.max(0) as usize;
        for (index, option) in self.options.iter().take(visible_rows).enumerate() {
            let y = rect.y + index as i64;
            let row = ViewRect {
                x: rect.x,
                y,
                width: rect.width,
                height: 1,
            };
            let selected = self.selected == Some(index);
            let focused = self.focused
                && self.focused_option == Some(index)
                && self.enabled
                && option.enabled;
            let enabled = self.enabled && option.enabled;
            let (fg, bg) = self.option_colors(enabled, focused);
            let text = radio_text(option, selected);
            let label_start = 4;
            let accelerator = accelerator_index(&option.label, option.accelerator)
                .map(|index| index + label_start);
            paint_chars(
                console,
                row,
                clip,
                truncated_chars(&text, row.width),
                |char_index| {
                    if !enabled {
                        self.style.disabled_fg
                    } else if Some(char_index) == accelerator && !focused {
                        self.style.accelerator_fg
                    } else {
                        fg
                    }
                },
                bg,
            );
        }
    }

    fn option_colors(&self, enabled: bool, focused: bool) -> (u8, u8) {
        if !enabled {
            (self.style.disabled_fg, self.style.bg)
        } else if focused {
            (self.style.active_fg, self.style.active_bg)
        } else {
            (self.style.fg, self.style.bg)
        }
    }

    fn option_is_selectable(&self, index: usize) -> bool {
        self.enabled && self.options.get(index).is_some_and(|option| option.enabled)
    }

    fn focus_step(&mut self, forward: bool) -> bool {
        if !self.enabled || self.options.is_empty() {
            return false;
        }
        let Some(target) = self.next_enabled_index(forward) else {
            return false;
        };
        self.set_focused_option(target)
    }

    fn next_enabled_index(&self, forward: bool) -> Option<usize> {
        let len = self.options.len();
        let start = self.focused_option.unwrap_or(0);
        (1..=len).find_map(|step| {
            let index = if forward {
                (start + step) % len
            } else {
                (start + len - (step % len)) % len
            };
            self.option_is_selectable(index).then_some(index)
        })
    }
}

fn radio_text(option: &RadioOption, selected: bool) -> String {
    let mark = if selected { '*' } else { ' ' };
    format!("({mark}) {}", option.label)
}
