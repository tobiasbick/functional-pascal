//! Project import and linking regression tests.

pub(super) use super::support::{load_and_build_program, toml_path, write_program_project_file};
pub(super) use super::{build_program, load_project};
pub(super) use fpas_parser::{Decl, DesignatorPart, Stmt};

mod errors;
mod graph;
mod short_names;
mod sources;
mod uses;
mod visibility;
