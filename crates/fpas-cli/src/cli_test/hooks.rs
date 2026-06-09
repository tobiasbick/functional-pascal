//! Discovery of project-level `Setup` / `Teardown` hooks for `fpas test`.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::fs;
use std::path::{Path, PathBuf};

use fpas_diagnostics::DiagnosticSeverity;
use fpas_parser::{CompilationUnit, Decl, QualifiedId, parse_compilation_unit};
use fpas_project::is_test_source_file;

/// One parameterless hook procedure in a test project unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TestHook {
    pub unit_name: String,
    pub procedure_name: String,
}

/// Optional setup/teardown hooks discovered from test project units.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct TestHooks {
    pub setup: Option<TestHook>,
    pub teardown: Option<TestHook>,
}

/// Scans project units for at most one `Setup` and one `Teardown` procedure each.
pub(super) fn discover_test_hooks(source_files: &[PathBuf]) -> Result<TestHooks, String> {
    let mut setup = None::<(PathBuf, TestHook)>;
    let mut teardown = None::<(PathBuf, TestHook)>;

    for path in source_files {
        if is_test_source_file(path) {
            continue;
        }

        let source = fs::read_to_string(path)
            .map_err(|error| format!("Error reading `{}`: {error}", path.display()))?;
        let (unit, errors) = parse_compilation_unit(&source);
        if errors
            .iter()
            .any(|diag| diag.as_diagnostic().severity == DiagnosticSeverity::Error)
        {
            continue;
        }

        let CompilationUnit::Unit(unit) = unit else {
            continue;
        };

        let unit_name = qualified_id_to_string(&unit.name);
        for decl in &unit.declarations {
            let Decl::Procedure(proc) = decl else {
                continue;
            };
            if !proc.type_params.is_empty() || !proc.params.is_empty() {
                continue;
            }

            if proc.name.eq_ignore_ascii_case("setup") {
                if let Some((existing, _)) = &setup {
                    return Err(duplicate_hook_error("Setup", existing, path));
                }
                setup = Some((
                    path.clone(),
                    TestHook {
                        unit_name: unit_name.clone(),
                        procedure_name: proc.name.clone(),
                    },
                ));
            } else if proc.name.eq_ignore_ascii_case("teardown") {
                if let Some((existing, _)) = &teardown {
                    return Err(duplicate_hook_error("Teardown", existing, path));
                }
                teardown = Some((
                    path.clone(),
                    TestHook {
                        unit_name: unit_name.clone(),
                        procedure_name: proc.name.clone(),
                    },
                ));
            }
        }
    }

    Ok(TestHooks {
        setup: setup.map(|(_, hook)| hook),
        teardown: teardown.map(|(_, hook)| hook),
    })
}

fn duplicate_hook_error(hook_name: &str, first: &Path, second: &Path) -> String {
    format!(
        "Test project defines `{hook_name}` in multiple units: `{}` and `{}`.\n  help: Provide at most one `{hook_name}` procedure across helper units.",
        first.display(),
        second.display()
    )
}

/// Builds a synthetic hook program that calls one project procedure.
pub(super) fn hook_program_source(hook: &TestHook) -> String {
    format!(
        "program __FpasTestHook;\nuses {unit};\nbegin\n  {proc}()\nend.",
        unit = hook.unit_name,
        proc = hook.procedure_name,
    )
}

fn qualified_id_to_string(id: &QualifiedId) -> String {
    id.parts.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn discover_test_hooks_finds_setup_and_teardown() {
        let dir = create_temp_dir("fpas-hooks-discover");
        write_text(
            &dir.join("fixture.fpas"),
            "unit Tests.Fixture;\nprocedure Setup();\nbegin end;\nprocedure Teardown();\nbegin end;",
        );
        write_text(
            &dir.join("demo_test.fpas"),
            "program D;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );

        let hooks = discover_test_hooks(&[dir.join("fixture.fpas"), dir.join("demo_test.fpas")])
            .expect("discover hooks");
        assert_eq!(
            hooks.setup.as_ref().map(|h| h.unit_name.as_str()),
            Some("Tests.Fixture")
        );
        assert_eq!(
            hooks.teardown.as_ref().map(|h| h.procedure_name.as_str()),
            Some("Teardown")
        );
    }

    #[test]
    fn discover_test_hooks_rejects_duplicate_setup() {
        let dir = create_temp_dir("fpas-hooks-dup");
        write_text(
            &dir.join("a.fpas"),
            "unit A.One;\nprocedure Setup();\nbegin end;",
        );
        write_text(
            &dir.join("b.fpas"),
            "unit B.Two;\nprocedure Setup();\nbegin end;",
        );

        let error = discover_test_hooks(&[dir.join("a.fpas"), dir.join("b.fpas")])
            .expect_err("duplicate setup");
        assert!(error.contains("multiple units"));
    }
}
