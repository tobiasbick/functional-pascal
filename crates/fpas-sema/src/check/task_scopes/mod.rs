//! Structured task scopes: lexical nesting and `go` ownership rules.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`.

mod context;
mod escapes;
mod handle_types;
mod ownership;

pub(crate) use context::TaskScopeContext;
