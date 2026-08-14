//! Shared ASCII-case-insensitive matching of executable routine names.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use fpas_bytecode::{FunctionId, VerifiedExecutable};

/// Return every executable function whose canonical or unique-short name matches `name`.
pub(in crate::vm::debug) fn matching_functions(
    executable: &VerifiedExecutable,
    name: &str,
) -> Vec<FunctionId> {
    executable
        .executable()
        .functions
        .iter()
        .enumerate()
        .filter(|(_, function)| {
            executable
                .executable()
                .strings
                .get(function.name)
                .is_some_and(|candidate| callable_name_matches(candidate, name))
        })
        .map(|(index, _)| FunctionId::new(u16::try_from(index).unwrap_or(u16::MAX)))
        .collect()
}

/// Whether a stored executable name matches a requested simple or qualified name.
pub(in crate::vm::debug) fn callable_name_matches(candidate: &str, requested: &str) -> bool {
    candidate.eq_ignore_ascii_case(requested)
        || (!requested.contains('.')
            && candidate
                .rsplit_once('.')
                .is_some_and(|(_, short)| short.eq_ignore_ascii_case(requested)))
}

#[cfg(test)]
mod tests {
    use super::{callable_name_matches, matching_functions};
    use fpas_bytecode::{
        CodeRange, Executable, FunctionDebugInfo, FunctionFlags, FunctionId, FunctionInfo,
        Instruction, InstructionAddress, Opcode, ReturnConvention, SourceId, SourceMap, SourceRun,
        StringId, StringTable, VerifiedExecutable,
    };

    fn catalog(names: &[&str]) -> VerifiedExecutable {
        let mut strings = vec!["root".to_string(), "test.fpas".to_string()];
        strings.extend(names.iter().map(|name| (*name).to_string()));
        let mut functions = vec![function(StringId::new(0), 0, 1)];
        let mut code = vec![Instruction::abc(Opcode::Return, u16::MAX, 0, 0, 0).expect("root")];
        for (index, _) in names.iter().enumerate() {
            let start = u32::try_from(code.len()).expect("start");
            code.push(Instruction::abc(Opcode::Return, u16::MAX, 0, 0, 0).expect("fn"));
            functions.push(function(
                StringId::new(u32::try_from(index.saturating_add(2)).expect("id")),
                start,
                start.saturating_add(1),
            ));
        }
        let runs = functions
            .iter()
            .map(|function| SourceRun {
                instruction_start: function.code.start,
                source: SourceId::new(0),
                line: 1,
                column: 3,
            })
            .collect();
        Executable {
            code,
            functions,
            constants: Vec::new(),
            strings: StringTable::new(strings),
            globals: Vec::new(),
            records: Vec::new(),
            enums: Vec::new(),
            enum_variants: Vec::new(),
            debug_types: vec![fpas_bytecode::DebugType::Unit],
            source_map: SourceMap {
                sources: vec![StringId::new(1)],
                runs,
            },
            entry: FunctionId::new(0),
        }
        .verify()
        .expect("routine catalog")
    }

    fn function(name: StringId, start: u32, end: u32) -> FunctionInfo {
        FunctionInfo {
            name,
            code: CodeRange::new(InstructionAddress::new(start), InstructionAddress::new(end)),
            arity: 0,
            capture_count: 0,
            register_count: 1,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: FunctionDebugInfo::default(),
        }
    }

    fn ids(executable: &VerifiedExecutable, name: &str) -> Vec<u16> {
        matching_functions(executable, name)
            .into_iter()
            .map(FunctionId::get)
            .collect()
    }

    #[test]
    fn canonical_qualified_short_and_mixed_case_matches_are_deterministic() {
        let executable = catalog(&["addtwo", "math.transform", "stats.transform"]);
        assert_eq!(ids(&executable, "addtwo"), vec![1]);
        assert_eq!(ids(&executable, "AddTwo"), vec![1]);
        assert_eq!(ids(&executable, "math.transform"), vec![2]);
        assert_eq!(ids(&executable, "Math.Transform"), vec![2]);
        assert_eq!(ids(&executable, "transform"), vec![2, 3]);
        assert!(ids(&executable, "missing").is_empty());
        assert!(ids(&executable, "math").is_empty());
        assert!(callable_name_matches("math.transform", "transform"));
        assert!(!callable_name_matches("math.transform", "math"));
    }
}
