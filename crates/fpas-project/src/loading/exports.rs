//! `[exports]` manifest validation for library projects.
//!
//! Documentation: `docs/pascal/program-structure/projects.md`

use crate::common::{
    SourceHeader, display_unit_key, qualified_id_to_string, read_source_header,
    validate_non_empty_entry,
};
use crate::loading::parse_cache::ParsedSourceCache;
use std::collections::HashSet;
use std::path::PathBuf;

/// Validates `exports.units` against library source files.
pub(crate) fn validate_library_exports(
    export_units: &[String],
    source_files: &[PathBuf],
    _parse_cache: &mut ParsedSourceCache,
) -> Result<HashSet<String>, String> {
    if export_units.is_empty() {
        return Err(
            "`exports.units` must contain at least one unit name.\n  help: List public units, for example `units = [\"MyLib.Core\"]`."
                .to_string(),
        );
    }

    let mut listed_units = HashSet::<String>::new();
    for raw in export_units {
        validate_non_empty_entry("exports.units", raw)?;
        let key = raw.trim().to_ascii_lowercase();
        if !listed_units.insert(key.clone()) {
            return Err(format!(
                "Duplicate export unit `{raw}` in `exports.units`.\n  help: List each unit name once."
            ));
        }
    }

    let mut defined_units = HashSet::<String>::new();
    for source_path in source_files {
        let (unit, _) = read_source_header(source_path, 0)?;
        let SourceHeader::Unit(unit) = unit else {
            continue;
        };
        defined_units.insert(qualified_id_to_string(&unit).to_ascii_lowercase());
    }

    for listed in &listed_units {
        if !defined_units.contains(listed) {
            let display = display_unit_key(listed);
            return Err(format!(
                "`exports.units` references unknown unit `{display}`.\n  help: Add a source file declaring `unit {display};` or fix the name."
            ));
        }
    }

    Ok(listed_units)
}
