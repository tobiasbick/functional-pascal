//! Lexical task-scope nesting of the routine body being checked.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

/// Task-scope state of one routine body.
#[derive(Debug, Default, Clone, Copy)]
struct RoutineFrame {
    /// Open `scope` statements of this routine body.
    depth: u32,
    /// The routine is anonymous and lexically inside a scope of its enclosing routine.
    anonymous_inside_scope: bool,
}

/// Tracks open task scopes per routine; ownership never crosses a routine boundary.
#[derive(Debug, Default)]
pub(crate) struct TaskScopeContext {
    current: RoutineFrame,
    enclosing: Vec<RoutineFrame>,
    next_routine_anonymous: bool,
}

impl TaskScopeContext {
    /// Marks the next routine body as an anonymous routine of the current body.
    pub(crate) fn prepare_anonymous_routine(&mut self) {
        self.next_routine_anonymous = true;
    }

    /// Starts a routine body with no open scopes of its own.
    pub(crate) fn enter_routine(&mut self) {
        let anonymous = std::mem::take(&mut self.next_routine_anonymous);
        let inside_scope = self.current.depth > 0 || self.current.anonymous_inside_scope;
        self.enclosing.push(self.current);
        self.current = RoutineFrame {
            depth: 0,
            anonymous_inside_scope: anonymous && inside_scope,
        };
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

    /// Whether a `go` here would silently escape a lexically enclosing scope (Q25).
    pub(crate) fn go_escapes_enclosing_scope(&self) -> bool {
        self.current.depth == 0 && self.current.anonymous_inside_scope
    }
}
