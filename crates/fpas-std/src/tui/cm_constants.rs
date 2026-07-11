//! Borland / Turbo Vision `CM_*` command identifiers for try-2 `Std.Tui`.
//!
//! Values are generated from the pinned `turbo_vision::core::command` dependency
//! by `crates/fpas-std/build.rs`. `CM_USER` remains project-local.
//!
//! **Documentation:** `docs/pascal/std/tui/app/types.md`

#![allow(
    dead_code,
    reason = "try-2 constants; sema exposes the supported Pascal subset"
)]

include!(concat!(env!("OUT_DIR"), "/cm_constants_generated.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_constants_keep_the_published_tui_values() {
        assert_eq!(CM_QUIT, 1);
        assert_eq!(CM_OK, 10);
        assert_eq!(CM_OPEN, 301);
        assert_eq!(CM_USER, 4096);
    }
}
