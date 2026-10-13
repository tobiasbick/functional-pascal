//! Lexical task-scope nesting of the routine body being checked.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

/// Task-scope state of one routine body.
#[derive(Debug, Default, Clone, Copy)]
struct RoutineFrame {
    /// Open `scope` statements of this routine body.
    depth: u32,
}

/// Tracks open task scopes per routine; ownership never crosses a routine boundary.
#[derive(Debug, Default)]
pub(crate) struct TaskScopeContext {
    current: RoutineFrame,
    enclosing: Vec<RoutineFrame>,
}

impl TaskScopeContext {
    /// Starts a routine body with no open scopes of its own.
    pub(crate) fn enter_routine(&mut self) {
        self.enclosing.push(self.current);
        self.current = RoutineFrame::default();
    }

    /// Restores the enclosing routine body.
    pub(crate) fn exit_routine(&mut self) {
        self.current = self.enclosing.pop().unwrap_or_default();
    }

    /// Opens a `scope` statement in the current routine body.
    pub(crate) fn enter_scope(&mut self) {
        self.current.depth += 1;
    }

    /// Closes the innermost `scope` statement of the current routine body.
    pub(crate) fn exit_scope(&mut self) {
        self.current.depth = self.current.depth.saturating_sub(1);
    }

    /// Whether a `go` here is owned by a scope of the current routine body.
    pub(crate) fn owns_go(&self) -> bool {
        self.current.depth > 0
    }
}
