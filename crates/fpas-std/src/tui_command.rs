//! Rust-internal command registry for the TUI application framework (Phase 7).
//!
//! Command shortcuts are resolved by the host before ordinary `OnKeyPressed`
//! dispatch. The Pascal-facing contract is documented in `docs/pascal/std/tui-app.md`.

use crate::ConsoleKeyEvent;

/// Application command identifier supplied by FPAS code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandBinding {
    key: ConsoleKeyEvent,
    command_id: CommandId,
}

/// Host-side keyboard shortcut registry for an active TUI session.
#[derive(Debug, Default)]
pub struct CommandRegistry {
    bindings: Vec<CommandBinding>,
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
            .find(|binding| binding.key == *key)
            .map(|binding| binding.command_id)
    }

    /// Remove all command bindings.
    pub fn clear(&mut self) {
        self.bindings.clear();
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
}
