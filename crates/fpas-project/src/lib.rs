//! Project loading and user-unit linking for Functional Pascal.
//!
//! Documentation:
//! - `docs/pascal/09-units.md`
//! - `docs/pascal/10-projects.md`

mod common;
mod link;
mod loading;
mod model;
mod paths;

pub use link::build_program;
pub use loading::load_project;
pub use model::{LoadedProject, ProjectKind};
