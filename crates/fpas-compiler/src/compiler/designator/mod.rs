//! Designator lowering (read/write paths, builtins, enum constants).
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` (from the repository root).

mod builtin_consts;
mod enum_consts;
mod read;
mod write;

use super::{Compiler, LocalRef};
