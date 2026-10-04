//! Shared in-memory bytecode images for compatible FPAS regression tests.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md)

mod compile;

#[cfg(test)]
use std::sync::Arc;

use super::parallel::PreparedTest;
use compile::{ImageBatch, ImageCandidate, compile_image_batches};

/// Compiles test entries before workers start and attaches their executable images.
///
/// Compilation failures deliberately leave tests untouched so
/// the normal single-test path can render the original diagnostic in isolation.
pub(super) fn attach_test_images(prepared: &mut [PreparedTest]) {
    let batches = prepared
        .iter()
        .enumerate()
        .map(|(prepared_index, test)| {
            ImageBatch::new(
                vec![ImageCandidate {
                    prepared_index,
                    path: test.path.clone(),
                }],
                test.link.clone(),
            )
        })
        .collect();

    for assignment in compile_image_batches(batches) {
        prepared[assignment.prepared_index].compiled = Some(assignment.compiled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_test::link::LinkContextCache;
    use crate::cli_test::run::{TestRunSettings, run_single_test_capture_prepared};
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn compatible_tests_are_precompiled_and_run_independently() {
        let dir = create_temp_dir("fpas-test-image");
        let first = dir.join("first_test.fpas");
        let second = dir.join("second_test.fpas");
        write_text(
            &first,
            r#"program First;  uses Std.Test as Test; begin Test.AssertEquals(2, 1 + 1); end program;"#,
        );
        write_text(
            &second,
            r#"program Second;  uses Std.Test as Test; begin Test.AssertTrue(true); end program;"#,
        );
        let mut prepared = vec![
            PreparedTest {
                index: 0,
                path: first,
                display: "first_test.fpas".to_string(),
                link: None,
                compiled: None,
            },
            PreparedTest {
                index: 1,
                path: second,
                display: "second_test.fpas".to_string(),
                link: None,
                compiled: None,
            },
        ];

        attach_test_images(&mut prepared);

        let first_image = prepared[0]
            .compiled
            .as_ref()
            .expect("first test must use image");
        let second_image = prepared[1]
            .compiled
            .as_ref()
            .expect("second test must use image");
        assert!(!Arc::ptr_eq(&first_image.image, &second_image.image));

        for test in &prepared {
            let (outcome, output) = run_single_test_capture_prepared(
                &test.path,
                None,
                TestRunSettings {
                    script_override: None,
                    timeout: None,
                    show_output: false,
                    diagnostics: crate::cli_output::DiagnosticFormat::Text,
                },
                test.compiled.as_ref(),
            );
            assert_eq!(
                outcome,
                crate::cli_test::report::TestOutcome::Pass,
                "{}",
                String::from_utf8_lossy(&output)
            );
        }
    }

    #[test]
    fn linked_tests_run_shared_unit_initialization_before_each_image_entry() {
        let dir = create_temp_dir("fpas-linked-test-image");
        let project = dir.join("suite.fpasprj");
        let helper = dir.join("helper.fpas");
        let first = dir.join("first_test.fpas");
        let second = dir.join("second_test.fpas");
        write_text(
            &project,
            "[project]\nname = \"suite\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
        );
        write_text(
            &helper,
            r#"unit Suite.Helper;
  public const Answer: integer := 42;
public function GetAnswer(): integer;
begin return Answer; end function;
end unit;

"#,
        );
        for (path, name) in [(&first, "First"), (&second, "Second")] {
            write_text(
                path,
                &format!(
                    "program {name}; uses Suite.Helper as Helper; uses Std.Test as Test; begin Test.AssertEquals(42, Helper.GetAnswer()); end program;"
                ),
            );
        }

        let mut links = LinkContextCache::new(None);
        let mut prepared = Vec::new();
        for (index, path) in [first, second].into_iter().enumerate() {
            let link = links
                .context_for_test(&path)
                .expect("project context must load");
            prepared.push(PreparedTest {
                index,
                display: path.to_string_lossy().into_owned(),
                path,
                link,
                compiled: None,
            });
        }

        attach_test_images(&mut prepared);

        for test in &prepared {
            let compiled = test.compiled.as_ref().expect("linked test must use image");
            let (outcome, _) = run_single_test_capture_prepared(
                &test.path,
                test.link.as_ref(),
                TestRunSettings {
                    script_override: None,
                    timeout: None,
                    show_output: false,
                    diagnostics: crate::cli_output::DiagnosticFormat::Text,
                },
                Some(compiled),
            );
            assert_eq!(outcome, crate::cli_test::report::TestOutcome::Pass);
        }
    }

    #[test]
    fn tests_with_module_level_declarations_are_precompiled() {
        let dir = create_temp_dir("fpas-test-image-declarations");
        let first = dir.join("first_test.fpas");
        let second = dir.join("second_test.fpas");
        for (path, name, value) in [(&first, "First", 1), (&second, "Second", 2)] {
            write_text(
                path,
                &format!(
                    r#"program {name}; uses Std.Test as Test; const Value: integer := {value}; begin Test.AssertEquals({value}, Value); end program;"#
                ),
            );
        }
        let mut prepared = [
            PreparedTest {
                index: 0,
                path: first,
                display: "first_test.fpas".to_string(),
                link: None,
                compiled: None,
            },
            PreparedTest {
                index: 1,
                path: second,
                display: "second_test.fpas".to_string(),
                link: None,
                compiled: None,
            },
        ];

        attach_test_images(&mut prepared);

        for test in &prepared {
            assert!(test.compiled.is_some());
            let (outcome, _) = run_single_test_capture_prepared(
                &test.path,
                None,
                TestRunSettings {
                    script_override: None,
                    timeout: None,
                    show_output: false,
                    diagnostics: crate::cli_output::DiagnosticFormat::Text,
                },
                test.compiled.as_ref(),
            );
            assert_eq!(outcome, crate::cli_test::report::TestOutcome::Pass);
        }
    }
}
