use super::{UnitFile, support::canonical_unit_key};
use crate::common::{parse_compilation_unit_file, qualified_id_to_string, validate_user_unit_name};

use fpas_lexer::DefineSet;
use fpas_parser::{CompilationUnit, Program};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn parse_program_file(path: &Path, defines: &DefineSet) -> Result<Program, String> {
    match parse_compilation_unit_file(path, defines)?.0 {
        CompilationUnit::Program(program) => Ok(program),
        CompilationUnit::Unit(unit) => Err(format!(
            "Expected a `program` file at `{}`, but found `unit {}`.",
            path.to_string_lossy(),
            qualified_id_to_string(&unit.name)
        )),
    }
}

pub(super) fn parse_unit_files(
    source_files: &[PathBuf],
    defines: &DefineSet,
) -> Result<HashMap<String, UnitFile>, String> {
    let mut by_unit = HashMap::<String, UnitFile>::new();

    for source_path in source_files {
        let unit = match parse_compilation_unit_file(source_path, defines)?.0 {
            CompilationUnit::Unit(unit) => unit,
            CompilationUnit::Program(program) => {
                return Err(format!(
                    "Source file `{}` declares `program {}`. Source files must use `unit` declarations.",
                    source_path.to_string_lossy(),
                    program.name
                ));
            }
        };
        validate_user_unit_name(source_path, &unit.name)?;

        let key = canonical_unit_key(&unit.name);
        if let Some(existing) = by_unit.get(&key) {
            return Err(format!(
                "Duplicate unit name `{}` found in `{}` and `{}`.\n  help: Use unique unit names across source files.",
                qualified_id_to_string(&unit.name),
                existing.path.to_string_lossy(),
                source_path.to_string_lossy()
            ));
        }

        by_unit.insert(
            key,
            UnitFile {
                path: source_path.clone(),
                unit,
            },
        );
    }

    Ok(by_unit)
}
