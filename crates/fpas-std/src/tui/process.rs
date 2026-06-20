//! Typed outcomes produced by one hosted TUI event-pump step.

/// Input category blocked by an active modal scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockedInput {
    /// Keyboard fallback was blocked.
    Key,
    /// Pointer fallback was blocked.
    Pointer,
    /// A command was blocked.
    Command,
}

/// Direction of a successful focus traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDirection {
    /// Forward Tab traversal.
    Forward,
    /// Backward Shift+Tab traversal.
    Backward,
}

/// Internal result of processing at most one hosted TUI event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessOutcome {
    /// No event was ready.
    Idle,
    /// Key fallback ran or was absent.
    Key { handled: bool, consumed: bool },
    /// Resize fallback ran or was absent.
    Resize { handled: bool },
    /// Pointer fallback ran or was absent.
    Pointer { handled: bool },
    /// Paste fallback ran or was absent.
    Paste { handled: bool },
    /// Terminal-focus-gained fallback ran or was absent.
    FocusGained { handled: bool },
    /// Terminal-focus-lost fallback ran or was absent.
    FocusLost { handled: bool },
    /// Focus traversal selected another view.
    FocusMoved(FocusDirection),
    /// A sourced command was handled or had no application callback.
    Command { handled: bool },
    /// Modal scope blocked fallback routing.
    Blocked(BlockedInput),
    /// A native widget consumed the event without dispatching a command.
    WidgetConsumed,
}

impl ProcessOutcome {
    /// Return whether this outcome represents completed event work.
    #[must_use]
    pub const fn did_work(self) -> bool {
        !matches!(self, Self::Idle)
    }

    /// Encode the temporary low-level bridge tag used by `HostProcessNext`.
    #[must_use]
    pub const fn bridge_tag(self) -> i64 {
        match self {
            Self::Idle => 0,
            Self::Key {
                handled: true,
                consumed: true,
            } => 1,
            Self::Resize { handled: true } => 2,
            Self::Key { handled: false, .. } => 3,
            Self::Resize { handled: false } => 4,
            Self::Pointer { handled: true } => 5,
            Self::Pointer { handled: false } => 7,
            Self::Paste { handled: true } => 8,
            Self::Paste { handled: false } => 9,
            Self::FocusGained { handled: true } => 10,
            Self::FocusGained { handled: false } => 11,
            Self::FocusLost { handled: true } => 12,
            Self::FocusLost { handled: false } => 13,
            Self::FocusMoved(FocusDirection::Forward) => 14,
            Self::FocusMoved(FocusDirection::Backward) => 15,
            Self::Command { handled: true } => 16,
            Self::Command { handled: false } => 17,
            Self::Blocked(BlockedInput::Key) => 18,
            Self::Blocked(BlockedInput::Pointer) => 19,
            Self::Blocked(BlockedInput::Command) => 20,
            Self::WidgetConsumed => 21,
            Self::Key {
                handled: true,
                consumed: false,
            } => 22,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_tags_are_confined_to_typed_outcome_conversion() {
        assert_eq!(ProcessOutcome::Idle.bridge_tag(), 0);
        assert_eq!(
            ProcessOutcome::FocusMoved(FocusDirection::Backward).bridge_tag(),
            15
        );
        assert_eq!(
            ProcessOutcome::Key {
                handled: true,
                consumed: false,
            }
            .bridge_tag(),
            22
        );
    }
}
