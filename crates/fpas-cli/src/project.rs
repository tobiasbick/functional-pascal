mod common;
mod link;
mod loading;
mod model;
mod paths;

use std::path::{Path, PathBuf};

pub use loading::load_project;
pub use model::{LoadedProject, ProjectKind};

pub fn build_program(
    main_path: &Path,
    source_files: &[PathBuf],
) -> Result<fpas_parser::Program, String> {
    link::build_program(main_path, source_files)
}

#[cfg(test)]
mod tests;
