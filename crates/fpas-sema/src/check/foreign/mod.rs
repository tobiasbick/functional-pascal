//! Recognition of spellings from other languages (Delphi, Free Pascal, Ada, C family) that only
//! name resolution can tell apart from valid FPAS: foreign built-in names and assignment to the
//! enclosing function's name.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

mod builtins;
mod constructors;
mod result_assignment;
mod types;

pub(crate) use types::{foreign_type_fix, foreign_type_hint};
