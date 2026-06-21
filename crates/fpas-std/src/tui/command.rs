//! Rust-internal command registry for the TUI application framework (Phase 7).
//!
//! Command shortcuts are resolved by the host before ordinary `OnKeyPressed`
//! dispatch. The Pascal-facing contract is documented in `docs/pascal/std/tui/app/README.md`.

use crate::ConsoleKeyEvent;
use crate::ViewId;
use std::collections::HashSet;

/// Application command identifier supplied by FPAS code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub i64);

/// Semantic category for a sourced TUI command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    /// Application-defined command binding or widget action.
    Application,
    /// Close the source view or active modal.
    Close,
    /// Zoom the source window.
    Zoom,
    /// Restore the source window from zoom.
    ZoomBack,
    /// Activate the next window root in z-order.
    NextWindow,
    /// Accept the active dialog.
    Accept,
    /// Cancel the active dialog.
    Cancel,
}

/// Command payload carrying both semantic kind and originating view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandEvent {
    /// Application command identifier.
    pub id: CommandId,
    /// View that produced or owned the command binding.
    pub source: Option<ViewId>,
    /// Built-in or application command category.
    pub kind: CommandKind,
}

impl CommandEvent {
    /// Construct an application-defined command event.
    #[must_use]
    pub const fn application(id: CommandId, source: Option<ViewId>) -> Self {
        Self {
            id,
            source,
            kind: CommandKind::Application,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandBinding {
    key: ConsoleKeyEvent,
    command_id: CommandId,
}

/// Host-side keyboard shortcut registry for an active TUI session.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandRegistry {
    bindings: Vec<CommandBinding>,
    disabled: HashSet<CommandId>,
}

impl CommandRegistry {
    /// Bind `key` to `command_id`.
    ///
    /// Rebinding the same key replaces the previous command.
    pub fn bind(&mut self, key: ConsoleKeyEvent, command_id: CommandId) {
        if let Some(binding) = self.bindings.iter_mut().find(|binding| binding.key == key) {
            binding.command_id = command_id;
        } else {
            self.bindings.push(CommandBinding { key, command_id });
        }
    }

    /// Resolve `key` to a command id, if one is bound.
    #[must_use]
    pub fn resolve(&self, key: &ConsoleKeyEvent) -> Option<CommandId> {
        self.bindings
            .iter()
            .find(|binding| binding.key == *key && self.is_enabled(binding.command_id))
            .map(|binding| binding.command_id)
    }

    /// Enable or disable every binding for `command_id`.
    pub fn set_enabled(&mut self, command_id: CommandId, enabled: bool) {
        if enabled {
            self.disabled.remove(&command_id);
        } else {
            self.disabled.insert(command_id);
        }
    }

    /// Return whether a command may currently resolve and dispatch.
    #[must_use]
    pub fn is_enabled(&self, command_id: CommandId) -> bool {
        !self.disabled.contains(&command_id)
    }

    /// Remove all command bindings.
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.disabled.clear();
    }

    /// Number of registered shortcuts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// True when no shortcuts are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_event::key_kind_index;

    fn key(name: &str, ch: char, ctrl: bool) -> ConsoleKeyEvent {
        ConsoleKeyEvent::new(key_kind_index(name), ch, false, ctrl, false, false)
    }

    #[test]
    fn resolve_returns_bound_command_id() {
        let mut commands = CommandRegistry::default();
        let key = key("Character", 's', true);
        commands.bind(key.clone(), CommandId(10));

        assert_eq!(commands.resolve(&key), Some(CommandId(10)));
    }

    #[test]
    fn bind_replaces_existing_key_binding() {
        let mut commands = CommandRegistry::default();
        let key = key("Character", 's', true);
        commands.bind(key.clone(), CommandId(10));
        commands.bind(key.clone(), CommandId(20));

        assert_eq!(commands.resolve(&key), Some(CommandId(20)));
    }

    #[test]
    fn resolve_requires_matching_modifiers() {
        let mut commands = CommandRegistry::default();
        commands.bind(key("Character", 's', true), CommandId(10));

        assert_eq!(commands.resolve(&key("Character", 's', false)), None);
    }

    #[test]
    fn clear_removes_all_bindings() {
        let mut commands = CommandRegistry::default();
        commands.bind(key("Character", 's', true), CommandId(10));
        commands.clear();

        assert!(commands.is_empty());
    }

    #[test]
    fn disabled_command_does_not_resolve_until_reenabled() {
        let mut commands = CommandRegistry::default();
        let key = key("Character", 's', true);
        commands.bind(key.clone(), CommandId(10));
        commands.set_enabled(CommandId(10), false);
        assert_eq!(commands.resolve(&key), None);

        commands.set_enabled(CommandId(10), true);
        assert_eq!(commands.resolve(&key), Some(CommandId(10)));
    }
}
