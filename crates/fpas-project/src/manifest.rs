//! Shared manifest reading and validation failures for `.fpasprj` and `.fpasworkspace`.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`,
//! `docs/pascal/tools/diagnostics.md`.

use std::path::Path;

use fpas_diagnostics::codes::{
    PROJECT_MANIFEST_READ_FAILED, PROJECT_MANIFEST_SYNTAX_INVALID, PROJECT_MANIFEST_VALUE_INVALID,
};
use serde::de::DeserializeOwned;

use crate::ProjectError;

/// Reads a manifest file; `kind` names it in the message (`project`, `workspace`).
pub(crate) fn read_manifest(path: &Path, kind: &str) -> Result<String, ProjectError> {
    std::fs::read_to_string(path).map_err(|error| {
        ProjectError::new(
            PROJECT_MANIFEST_READ_FAILED,
            format!(
                "Error reading {kind} file `{}`: {error}",
                path.to_string_lossy()
            ),
        )
        .with_help("Check that the manifest file exists and is readable.")
        .in_file(path)
    })
}

/// Parses manifest TOML into its schema, explaining the expected layout in `help`.
pub(crate) fn parse_manifest<T: DeserializeOwned>(
    path: &Path,
    kind: &str,
    text: &str,
    help: &str,
) -> Result<T, ProjectError> {
    toml::from_str(text).map_err(|error| {
        ProjectError::new(
            PROJECT_MANIFEST_SYNTAX_INVALID,
            format!(
                "Invalid {kind} file `{}`: {}",
                path.to_string_lossy(),
                error.to_string().trim_end()
            ),
        )
        .with_help(help)
        .in_file(path)
    })
}

/// Rejects a manifest field value that is missing, empty or disallowed.
pub(crate) fn invalid_value(message: impl Into<String>, help: &str) -> ProjectError {
    ProjectError::new(PROJECT_MANIFEST_VALUE_INVALID, message).with_help(help)
}
