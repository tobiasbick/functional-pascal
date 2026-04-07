//! Designator lowering (read/write paths, builtins, enum constants).
//!
//! **Documentation:** `docs/pascal/02-basics.md` (from the repository root).

mod builtin_consts;
mod enum_consts;
mod read;
mod write;

use super::{Compiler, LocalRef};
