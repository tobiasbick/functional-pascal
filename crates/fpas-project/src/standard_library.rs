//! Discovery of trusted, source-level standard-library units.

use std::path::{Path, PathBuf};

/// Source files loaded from the implementation-owned standard-library root.
#[derive(Debug, Clone)]
pub struct StandardLibrary {
    source_files: Vec<PathBuf>,
}

impl StandardLibrary {
    /// Returns the standard-library unit source files in stable path order.
    pub fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }
}

/// Loads every FPAS unit below an implementation-owned library root.
pub fn load_standard_library(root: &Path) -> Result<StandardLibrary, String> {
    if !root.is_dir() {
        return Err(format!(
            "Standard library directory `{}` does not exist.\n  help: Pass `--std-lib <directory>` containing the `Std` directory.",
            root.display()
        ));
    }

    let mut source_files = Vec::new();
    collect_fpas_files(root, &mut source_files)?;
    source_files.sort();
    Ok(StandardLibrary { source_files })
}

fn collect_fpas_files(dir: &Path, target: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|error| {
        format!(
            "Error reading standard library directory `{}`: {error}",
            dir.display()
        )
    })? {
        let path = entry
            .map_err(|error| {
                format!(
                    "Error reading standard library directory `{}`: {error}",
                    dir.display()
                )
            })?
            .path();
        if path.is_dir() {
            collect_fpas_files(&path, target)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("fpas"))
        {
            target.push(path);
        }
    }
    Ok(())
}
