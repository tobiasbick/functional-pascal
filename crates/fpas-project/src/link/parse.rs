use super::{
    UnitFile,
    source_map::{apply_program_source_id, apply_unit_source_id},
    support::canonical_unit_key,
};
use crate::common::{parse_compilation_unit_file, qualified_id_to_string, validate_user_unit_name};

use fpas_parser::{CompilationUnit, Program};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn parse_program_file(path: &Path) -> Result<Program, String> {
    match parse_compilation_unit_file(path, 0)?.0 {
        CompilationUnit::Program(mut program) => {
            apply_program_source_id(&mut program, 0);
            Ok(program)
        }
        CompilationUnit::Unit(unit) => Err(format!(
            "Expected a `program` file at `{}`, but found `unit {}`.",
            path.to_string_lossy(),
            qualified_id_to_string(&unit.name)
        )),
    }
}

pub(super) fn parse_unit_files(
    source_files: &[PathBuf],
    source_paths: &mut Vec<PathBuf>,
) -> Result<HashMap<String, UnitFile>, String> {
    let mut by_unit = HashMap::<String, UnitFile>::new();

    for source_path in source_files {
        let source_id = next_source_id(source_paths.len())?;
        source_paths.push(source_path.clone());

        let mut unit = match parse_compilation_unit_file(source_path, source_id)?.0 {
            CompilationUnit::Unit(unit) => unit,
            CompilationUnit::Program(program) => {
                return Err(format!(
                    "Source file `{}` declares `program {}`. Source files must use `unit` declarations.",
                    source_path.to_string_lossy(),
                    program.name
                ));
            }
        };
        apply_unit_source_id(&mut unit, source_id);
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

fn next_source_id(source_path_count: usize) -> Result<u32, String> {
    u32::try_from(source_path_count).map_err(|_| {
        format!(
            "Too many source files in project: {source_path_count}.
  help: Reduce the number of linked source files so source IDs fit into 32 bits."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::next_source_id;

    #[test]
    fn next_source_id_rejects_counts_that_do_not_fit_into_u32() {
        let result = next_source_id((u32::MAX as usize).saturating_add(1));

        assert!(
            result.is_err(),
            "overflowing source path counts must be rejected"
        );
        let error = result.err().unwrap_or_default();

        assert!(error.contains("Too many source files in project"));
    }
}
