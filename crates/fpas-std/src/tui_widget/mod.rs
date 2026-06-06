//! Rust-hosted TUI widgets painted directly by the VM host.
//!
//! Plan: `docs/future/tui-application-framework.md`
//! Spec: `docs/pascal/std/tui-app.md`

mod solid_fill;

pub use solid_fill::{SolidFillWidget, ViewWidget};
