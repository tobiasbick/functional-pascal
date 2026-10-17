//! Recognition of spellings from other languages (Delphi, Free Pascal, Ada, C family) that only
//! name resolution can tell apart from valid FPAS: foreign built-in names and assignment to the
//! enclosing function's name.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

mod builtins;
mod result_assignment;

pub(crate) use builtins::foreign_builtin_hint;
