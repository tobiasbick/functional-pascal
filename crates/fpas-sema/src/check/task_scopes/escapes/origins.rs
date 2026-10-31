//! Where a tracked value comes from and which bindings may hold it.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md#handle-escapes`.

use std::collections::HashMap;

use fpas_lexer::Span;

/// Source of a value that must not outlive its region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    /// Handle of a child owned by the task scope at this 1-based nesting depth.
    Scope {
        /// Nesting depth of the owning scope in the current routine body.
        depth: u32,
        /// Source span of the owning `scope` statement.
        span: Span,
    },
    /// Value derived from the parameter at this position of the current routine.
    Parameter(usize),
    /// A callable supplied by a routine's caller.
    CallableParameter(usize),
    /// A resource-bearing parameter retained by a callback value.
    BorrowedCapture(usize),
    /// A newly owned value whose validity depends on a borrowed parameter.
    DependencyParameter(usize),
    /// Resource owned by the lexical frame that declared a using binding.
    Resource { frame: usize, span: Span },
    /// A separately owned resource whose validity depends on another resource.
    Dependency { frame: usize, span: Span },
}

/// Origins of one value; usually empty.
pub(super) type Owners = Vec<Origin>;

/// Adds origins without duplicates.
pub(super) fn merge(target: &mut Owners, extra: impl IntoIterator<Item = Origin>) {
    for origin in extra {
        if !target.contains(&origin) {
            target.push(origin);
        }
    }
}

/// One local binding of the analyzed routine body.
#[derive(Debug, Clone, Default)]
pub(super) struct Binding {
    /// Origins of values the binding may hold.
    pub(super) owners: Owners,
    /// Task-scope depth at the declaration.
    pub(super) depth: u32,
    /// Storage outside the routine body, such as a `var` parameter or a captured binding.
    pub(super) outer: bool,
}

impl Binding {
    /// Whether `origin` may be stored in this binding without leaving its region.
    pub(super) fn may_hold(&self, origin: Origin) -> bool {
        !self.outer
            && match origin {
                Origin::Scope { depth, .. } => self.depth >= depth,
                Origin::Parameter(_) => true,
                Origin::CallableParameter(_) => true,
                Origin::BorrowedCapture(_) => true,
                Origin::DependencyParameter(_) => true,
                Origin::Resource { .. } | Origin::Dependency { .. } => true,
            }
    }
}

/// One lexical level: local bindings and the named routines declared there.
#[derive(Debug, Default)]
pub(super) struct Frame {
    pub(super) bindings: HashMap<String, Binding>,
    pub(super) routines: HashMap<String, usize>,
    /// Proven non-retaining local callback values.
    pub(super) safe_callbacks: std::collections::HashSet<String>,
    /// Entering an anonymous routine body: enclosing bindings become outer storage.
    pub(super) routine_boundary: bool,
}
