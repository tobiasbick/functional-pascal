//! Optional `[test]` manifest overrides for `fpas test`.
//!
//! Spec: [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::model::ProjectKind;
use crate::paths::resolve_explicit_file_path;
use crate::test_sources::is_test_source_file;

/// Per-test runner settings from a project manifest.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TestFileOverride {
    /// Optional script path relative to the project root.
    pub script: Option<PathBuf>,
    /// When set, overrides `[config] headless_graph` from the loaded script.
    pub headless_graph: Option<bool>,
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
struct TestFileOverrideRaw {
    script: Option<String>,
    headless_graph: Option<bool>,
}

/// Parses and validates an optional `[test]` section for the given project sources.
pub(super) fn parse_test_section(
    kind: ProjectKind,
    section: Option<TestSectionRaw>,
    source_files: &[PathBuf],
    project_root: &Path,
    project_path: &Path,
) -> Result<TestManifest, String> {
    let Some(section) = section else {
        return Ok(TestManifest::default());
    };

    if !matches!(kind, ProjectKind::Test) {
        return Err(format!(
            "Project `{}` must not define `[test]`.\n  help: `[test]` overrides are only allowed in `kind = \"test\"` projects.",
            project_path.to_string_lossy()
        ));
    }

    if section.overrides.is_empty() {
        return Ok(TestManifest::default());
    }

    let mut overrides = HashMap::new();
    for (key, raw) in section.overrides {
        let normalized = key.trim();
        if normalized.is_empty() {
            return Err(
                "`[test.overrides]` contains an empty test file key.\n  help: Use a basename such as `menu_test.fpas`."
                    .to_string(),
            );
        }

        let key_path = PathBuf::from(normalized);
        if !is_test_source_file(&key_path) {
            return Err(format!(
                "`[test.overrides]` key `{normalized}` is not a test file name.\n  help: Keys must end with `_test.fpas`."
            ));
        }

        let lookup_key = normalized.to_ascii_lowercase();
        if !source_files.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(normalized))
        }) {
            return Err(format!(
                "`[test.overrides]` key `{normalized}` does not match any project source file.\n  help: Add the test file to `[sources].include` first."
            ));
        }

        if raw.script.is_none() && raw.headless_graph.is_none() {
            return Err(format!(
                "`[test.overrides.{normalized}]` must set `script` and/or `headless_graph`.\n  help: Remove empty override tables or add at least one field."
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

        if overrides
            .insert(
                lookup_key,
                TestFileOverride {
                    script,
                    headless_graph: raw.headless_graph,
                },
            )
            .is_some()
        {
            return Err(format!(
                "Duplicate `[test.overrides]` entry for `{normalized}`.\n  help: Define each test file override once."
            ));
        }
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
                    script: None,
                    headless_graph: Some(true),
                },
            )]),
        };

        assert_eq!(
            manifest
                .override_for(Path::new("dir/ALPHA_test.fpas"))
                .and_then(|value| value.headless_graph),
            Some(true)
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

        assert!(matches!(result, Err(error) if error.contains("must not define `[test]`")));
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
                            script: None,
                            headless_graph: Some(true),
                        },
                    ),
                    (
                        "ALPHA_TEST.FPAS".to_string(),
                        TestFileOverrideRaw {
                            script: None,
                            headless_graph: Some(false),
                        },
                    ),
                ]),
            }),
            &[source],
            Path::new("."),
            Path::new("tests.fpasprj"),
        );

        assert!(
            matches!(result, Err(error) if error.contains("Duplicate `[test.overrides]` entry"))
        );
    }
}
