//! Higher-order `Std.Array` semantic checks.
//!
//! **Documentation:** `docs/pascal/std/collections/array.md` (from the repository root).
//!
//! - [`transform`]: `Map`, `Filter`, `Reduce`, `FlatMap`
//! - [`search`]: `Find`, `FindIndex`, `Any`, `All`, `ForEach`

mod callbacks;
mod search;
mod transform;

pub(super) use search::{check_all, check_any, check_find, check_find_index, check_for_each};
pub(super) use transform::{check_filter, check_flat_map, check_map, check_reduce};
