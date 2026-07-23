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
    use crate::cli_test::run::run_single_test_capture_prepared;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn compatible_tests_are_precompiled_and_run_independently() {
        let dir = create_temp_dir("fpas-test-image");
        let first = dir.join("first_test.fpas");
        let second = dir.join("second_test.fpas");
        write_text(
            &first,
            "program First; uses Std.Test; begin AssertEquals(2, 1 + 1) end.",
        );
        write_text(
            &second,
            "program Second; uses Std.Test; begin AssertTrue(true) end.",
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
                None,
                None,
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
            "unit Suite.Helper;\nvar Answer: integer := 42;\nfunction GetAnswer(): integer;\nbegin return Answer end;\n",
        );
        for (path, name) in [(&first, "First"), (&second, "Second")] {
            write_text(
                path,
                &format!(
                    "program {name}; uses Suite.Helper, Std.Test; begin AssertEquals(42, GetAnswer()) end."
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
                None,
                None,
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
                    "program {name}; uses Std.Test; var Value: integer := {value}; begin AssertEquals({value}, Value) end."
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
                None,
                None,
                test.compiled.as_ref(),
            );
            assert_eq!(outcome, crate::cli_test::report::TestOutcome::Pass);
        }
    }
}
