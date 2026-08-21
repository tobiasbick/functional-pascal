//! Test source file naming and validation for `kind = "test"` projects.
//!
//! **Documentation:** `docs/pascal/program-structure/projects.md`, `docs/pascal/std/testing/test.md`.

use std::path::Path;

use fpas_parser::CompilationUnit;

use crate::loading::parse_cache::ParsedSourceCache;
use crate::source::{qualified_id_to_string, validate_user_unit_name};
use std::collections::HashMap;
use std::path::PathBuf;

/// Returns true when `path` ends with `_test.fpas` (case-insensitive).
pub fn is_test_source_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.len() > "_test.fpas".len() && name.to_ascii_lowercase().ends_with("_test.fpas")
        })
}

/// Validates unit sources and `*_test.fpas` program entry files for test projects.
pub(crate) fn validate_project_test_sources(
    source_files: Vec<PathBuf>,
    warnings: &mut Vec<String>,
    parse_cache: &mut ParsedSourceCache,
) -> Result<Vec<PathBuf>, String> {
    let mut validated = Vec::new();
    let mut seen_unit_names = HashMap::<String, PathBuf>::new();

    for source_path in source_files {
        let (unit, parse_warnings) = parse_cache.parse(&source_path, 0)?;
        warnings.extend(parse_warnings);

        match unit {
            CompilationUnit::Program(program) => {
                if is_test_source_file(&source_path) {
                    validated.push(source_path);
                } else {
                    warnings.push(format!(
                        "Source file `{}` declares `program {}` and was skipped. Test projects keep only `*_test.fpas` program files and `unit` sources.",
                        source_path.to_string_lossy(),
                        program.name
                    ));
                }
            }
            CompilationUnit::Unit(unit) => {
                validate_user_unit_name(&source_path, &unit.name)?;
                let unit_name = qualified_id_to_string(&unit.name);
                let key = unit_name.to_ascii_lowercase();
                if let Some(first_path) = seen_unit_names.get(&key) {
                    return Err(format!(
                        "Duplicate unit name `{unit_name}` found in `{}` and `{}`.\n  help: Use a unique `unit` namespace per source file.",
                        first_path.to_string_lossy(),
                        source_path.to_string_lossy()
                    ));
                }
                seen_unit_names.insert(key, source_path.clone());
                validated.push(source_path);
            }
        }
    }

    Ok(validated)
}

#[cfg(test)]
mod tests {
    use super::is_test_source_file;
    use std::path::Path;

    #[test]
    fn recognizes_test_file_names_case_insensitively() {
        assert!(is_test_source_file(Path::new("feature_TEST.FPAS")));
    }

    #[test]
    fn rejects_the_suffix_without_a_test_name() {
        assert!(!is_test_source_file(Path::new("_test.fpas")));
    }

    #[test]
    fn rejects_non_test_pascal_file_names() {
        assert!(!is_test_source_file(Path::new("feature.fpas")));
    }

    #[test]
    fn rejects_test_suffixes_with_a_different_extension() {
        assert!(!is_test_source_file(Path::new("feature_test.pas")));
    }
}
