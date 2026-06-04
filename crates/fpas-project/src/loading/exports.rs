//! `[exports]` manifest validation for library projects.
//!
//! Documentation: `docs/pascal/10-projects.md`

use crate::common::{parse_compilation_unit_file, qualified_id_to_string};
use fpas_parser::CompilationUnit;
use std::collections::HashSet;
use std::path::PathBuf;

/// Parsed `[exports]` section for a library project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LibraryExports {
    /// Unit names listed in `exports.units` (ASCII-lowercase keys).
    pub listed_units: HashSet<String>,
}

/// Validates `exports.units` against library source files.
pub(crate) fn validate_library_exports(
    export_units: &[String],
    source_files: &[PathBuf],
) -> Result<LibraryExports, String> {
    if export_units.is_empty() {
        return Err(
            "`exports.units` must contain at least one unit name.\n  help: List public units, for example `units = [\"MyLib.Core\"]`."
                .to_string(),
        );
    }

    let mut listed_units = HashSet::<String>::new();
    for raw in export_units {
        validate_non_empty("exports.units", raw)?;
        let key = raw.trim().to_ascii_lowercase();
        if !listed_units.insert(key.clone()) {
            return Err(format!(
                "Duplicate export unit `{raw}` in `exports.units`.\n  help: List each unit name once."
            ));
        }
    }

    let mut defined_units = HashSet::<String>::new();
    for source_path in source_files {
        let (unit, _) = parse_compilation_unit_file(source_path, 0)?;
        let CompilationUnit::Unit(unit) = unit else {
            continue;
        };
        defined_units.insert(qualified_id_to_string(&unit.name).to_ascii_lowercase());
    }

    for listed in &listed_units {
        if !defined_units.contains(listed) {
            let display = display_unit_key(listed);
            return Err(format!(
                "`exports.units` references unknown unit `{display}`.\n  help: Add a source file declaring `unit {display};` or fix the name."
            ));
        }
    }

    Ok(LibraryExports { listed_units })
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "A `{field_name}` entry is empty.\n  help: Remove empty entries or provide a valid unit name."
        ));
    }
    Ok(())
}

fn display_unit_key(key: &str) -> String {
    let mut result = String::new();
    for (index, segment) in key.split('.').enumerate() {
        if index > 0 {
            result.push('.');
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }
    result
}
