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

pub use link::{build_program, build_program_with_defines};
pub use loading::{load_project, load_project_with_defines};
pub use model::{LoadedProject, ProjectKind};
