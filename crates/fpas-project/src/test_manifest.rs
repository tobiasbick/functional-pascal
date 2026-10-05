//! Optional `[test]` manifest overrides for `fpas test`.
//!
//! Spec: [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use fpas_diagnostics::codes::PROJECT_DUPLICATE_ENTRY;
use serde::Deserialize;

use crate::ProjectError;
use crate::manifest::invalid_value;
use crate::model::ProjectKind;
use crate::paths::resolve_explicit_file_path;
use crate::test_sources::is_test_source_file;

/// Per-test runner settings from a project manifest.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestFileOverride {
    /// Optional script path relative to the project root.
    pub script: Option<PathBuf>,
}

/// Parsed `[test]` section for a `kind = "test"` project.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestManifest {
    overrides: HashMap<String, TestFileOverride>,
}

impl TestManifest {
    /// Returns overrides keyed by test file basename (case-insensitive).
    pub fn override_for(&self, test_path: &Path) -> Option<&TestFileOverride> {
        test_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_ascii_lowercase())
            .and_then(|key| self.overrides.get(&key))
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct TestSectionRaw {
    #[serde(default)]
    overrides: HashMap<String, TestFileOverrideRaw>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TestFileOverrideRaw {
    script: Option<String>,
}

/// Parses and validates an optional `[test]` section for the given project sources.
pub(super) fn parse_test_section(
    kind: ProjectKind,
    section: Option<TestSectionRaw>,
    source_files: &[PathBuf],
    project_root: &Path,
    project_path: &Path,
) -> Result<TestManifest, ProjectError> {
    let Some(section) = section else {
        return Ok(TestManifest::default());
    };

    if !matches!(kind, ProjectKind::Test) {
        return Err(invalid_value(
            format!(
                "Project `{}` must not define `[test]`.",
                project_path.to_string_lossy()
            ),
            "`[test]` overrides are only allowed in `kind = \"test\"` projects.",
        ));
    }

    if section.overrides.is_empty() {
        return Ok(TestManifest::default());
    }

    let mut normalized_names = HashSet::with_capacity(section.overrides.len());
    for key in section.overrides.keys() {
        let normalized = key.trim();
        if !normalized_names.insert(normalized.to_ascii_lowercase()) {
            return Err(ProjectError::new(
                PROJECT_DUPLICATE_ENTRY,
                format!("Duplicate `[test.overrides]` entry for `{normalized}`."),
            )
            .with_help("Define each test file override once."));
        }
    }

    let mut overrides = HashMap::new();
    for (key, raw) in section.overrides {
        let normalized = key.trim();
        if normalized.is_empty() {
            return Err(invalid_value(
                "`[test.overrides]` contains an empty test file key.".to_string(),
                "Use a basename such as `menu_test.fpas`.",
            ));
        }

        let key_path = PathBuf::from(normalized);
        if !is_test_source_file(&key_path) {
            return Err(invalid_value(
                format!("`[test.overrides]` key `{normalized}` is not a test file name."),
                "Keys must end with `_test.fpas`.",
            ));
        }

        let lookup_key = normalized.to_ascii_lowercase();
        if !source_files.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(normalized))
        }) {
            return Err(invalid_value(
                format!(
                    "`[test.overrides]` key `{normalized}` does not match any project source file."
                ),
                "Add the test file to `[sources].include` first.",
            ));
        }

        if raw.script.is_none() {
            return Err(invalid_value(
                format!("`[test.overrides.{normalized}]` must set `script`."),
                "Remove empty override tables or add a script path.",
            ));
        }

        let script = match raw.script {
            None => None,
            Some(path) => Some(resolve_explicit_file_path(
                &format!("[test.overrides.{normalized}].script"),
                &path,
                project_root,
            )?),
        };

        overrides.insert(lookup_key, TestFileOverride { script });
    }

    Ok(TestManifest { overrides })
}

#[cfg(test)]
mod tests {
    use super::{
        TestFileOverride, TestFileOverrideRaw, TestManifest, TestSectionRaw, parse_test_section,
    };
    use crate::ProjectKind;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn override_for_matches_basename_case_insensitively() {
        let manifest = TestManifest {
            overrides: HashMap::from([(
                "alpha_test.fpas".to_string(),
                TestFileOverride {
                    script: Some(PathBuf::from("alpha.script.toml")),
                },
            )]),
        };

        assert_eq!(
            manifest
                .override_for(Path::new("dir/ALPHA_test.fpas"))
                .and_then(|value| value.script.as_deref()),
            Some(Path::new("alpha.script.toml"))
        );
    }

    #[test]
    fn empty_test_section_has_no_overrides() {
        let result = parse_test_section(
            ProjectKind::Test,
            Some(TestSectionRaw {
                overrides: HashMap::new(),
            }),
            &[],
            Path::new("."),
            Path::new("tests.fpasprj"),
        );

        assert_eq!(result, Ok(TestManifest::default()));
    }

    #[test]
    fn test_overrides_are_rejected_for_non_test_projects() {
        let result = parse_test_section(
            ProjectKind::Program,
            Some(TestSectionRaw {
                overrides: HashMap::new(),
            }),
            &[],
            Path::new("."),
            Path::new("demo.fpasprj"),
        );

        assert!(
            matches!(result, Err(error) if error.to_string().contains("must not define `[test]`"))
        );
    }

    #[test]
    fn duplicate_override_names_differing_only_by_case_are_rejected() {
        let source = PathBuf::from("alpha_test.fpas");
        let result = parse_test_section(
            ProjectKind::Test,
            Some(TestSectionRaw {
                overrides: HashMap::from([
                    (
                        "alpha_test.fpas".to_string(),
                        TestFileOverrideRaw {
                            script: Some("alpha.script.toml".to_string()),
                        },
                    ),
                    (
                        "ALPHA_TEST.FPAS".to_string(),
                        TestFileOverrideRaw {
                            script: Some("alpha.script.toml".to_string()),
                        },
                    ),
                ]),
            }),
            &[source],
            Path::new("."),
            Path::new("tests.fpasprj"),
        );

        assert!(
            matches!(result, Err(error) if error.to_string().contains("Duplicate `[test.overrides]` entry"))
        );
    }
}
