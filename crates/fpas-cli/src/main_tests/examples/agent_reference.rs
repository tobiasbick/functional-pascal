//! Compile and execute the literal examples in `docs/pascal/agent-reference.md`.

use super::{handbook::code_blocks, repo_root, support};
use crate::test_support::{
    create_temp_dir, write_library_fpasprj, write_program_fpasprj, write_text,
};
use fpas_parser::CompilationUnit;
use std::collections::HashSet;
use std::fs;

#[test]
fn agent_reference_examples_check_and_run() {
    let root = repo_root();
    let markdown = fs::read_to_string(root.join("docs/pascal/agent-reference.md"))
        .expect("read compact reference");
    let blocks = code_blocks(&markdown);
    assert!(!blocks.is_empty(), "reference must contain examples");
    let dir = create_temp_dir("agent-reference");
    let mut names = HashSet::new();
    let mut examples = Vec::new();
    // Write all sources before checking any: programs may import the documented units.
    for block in blocks {
        assert_eq!(
            block.language, "pascal",
            "line {}: every fence must be a tested Pascal source",
            block.line
        );
        let (unit, diagnostics) = fpas_parser::parse_compilation_unit(&block.source);
        assert!(
            diagnostics.is_empty(),
            "line {}: {diagnostics:?}",
            block.line
        );
        let (name, runnable) = match unit {
            CompilationUnit::Program(program) => (program.name, true),
            CompilationUnit::Unit(unit) => (unit.name.parts.join("."), false),
        };
        assert!(
            names.insert(name.to_ascii_lowercase()),
            "line {}: duplicate source name {name}",
            block.line
        );
        let path = dir.join(format!("{name}.fpas"));
        write_text(&path, &block.source);
        examples.push((block.line, path, runnable));
    }
    let unit_files: Vec<_> = examples
        .iter()
        .filter(|(_, _, runnable)| !runnable)
        .map(|(_, path, _)| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    let includes: Vec<_> = unit_files.iter().map(String::as_str).collect();
    let mut failures = Vec::new();
    for (line, path, runnable) in examples {
        let project = path.with_extension("fpasprj");
        if runnable {
            write_program_fpasprj(
                &project,
                path.file_name().unwrap().to_str().unwrap(),
                &includes,
            );
        } else {
            write_library_fpasprj(&project, &includes);
        }
        let commands: &[&str] = if runnable {
            &["check", "run"]
        } else {
            &["check"]
        };
        for command in commands {
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
                &[
                    (*command).into(),
                    "--std-lib".into(),
                    root.join("lib").to_string_lossy().into_owned(),
                    project.to_string_lossy().into_owned(),
                ],
                &root,
            );
            if exit != 0 || !stderr.is_empty() {
                failures.push(format!(
                    "line {line}: {command} {} (exit {exit})\n{stdout}{stderr}",
                    path.display()
                ));
                break;
            }
        }
    }
    fs::remove_dir_all(dir).expect("remove reference fixtures");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
