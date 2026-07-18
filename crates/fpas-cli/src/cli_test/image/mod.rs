//! Shared in-memory bytecode images for compatible FPAS regression tests.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md)

mod compile;

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;

use fpas_diagnostics::DiagnosticSeverity;
use fpas_parser::{CompilationUnit, parse_compilation_unit};

use super::parallel::PreparedTest;
use compile::{ImageBatch, ImageCandidate, compile_image_batches};

const MAX_TESTS_PER_IMAGE: usize = 96;

#[derive(PartialEq, Eq, Hash)]
struct ImageGroupKey {
    context: Option<PathBuf>,
    uses: Vec<String>,
}

/// Compiles compatible tests in bounded groups and attaches shared bytecode entries.
///
/// Loading or bundle compilation failures deliberately leave tests untouched so
/// the normal single-test path can render the original diagnostic in isolation.
pub(super) fn attach_test_images(prepared: &mut [PreparedTest]) {
    let mut groups = HashMap::<ImageGroupKey, VecDeque<ImageCandidate>>::new();

    for (prepared_index, test) in prepared.iter().enumerate() {
        if test
            .link
            .as_ref()
            .is_some_and(|link| link.hooks.setup.is_some() || link.hooks.teardown.is_some())
        {
            continue;
        }
        let Some(uses) = bundle_candidate_uses(&test.path) else {
            continue;
        };
        let key = ImageGroupKey {
            context: test
                .link
                .as_ref()
                .and_then(|link| link.bundle_context.clone()),
            uses,
        };
        groups.entry(key).or_default().push_back(ImageCandidate {
            prepared_index,
            path: test.path.clone(),
        });
    }

    let mut batches = Vec::new();
    for mut candidates in groups.into_values() {
        while candidates.len() >= 2 {
            let batch_len = next_batch_len(candidates.len());
            let batch = candidates.drain(..batch_len).collect::<Vec<_>>();
            let link = prepared[batch[0].prepared_index].link.clone();
            batches.push(ImageBatch::new(batch, link));
        }
    }

    for assignment in compile_image_batches(batches) {
        prepared[assignment.prepared_index].compiled = Some(assignment.compiled);
    }
}

fn next_batch_len(remaining: usize) -> usize {
    if remaining <= MAX_TESTS_PER_IMAGE {
        return remaining;
    }
    if remaining % MAX_TESTS_PER_IMAGE == 1 {
        return MAX_TESTS_PER_IMAGE - 1;
    }
    MAX_TESTS_PER_IMAGE
}

fn bundle_candidate_uses(path: &Path) -> Option<Vec<String>> {
    let source = fs::read_to_string(path).ok()?;
    let (unit, diagnostics) = parse_compilation_unit(&source);
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.as_diagnostic().severity == DiagnosticSeverity::Error)
    {
        return None;
    }
    let CompilationUnit::Program(program) = unit else {
        return None;
    };
    if !program.declarations.is_empty() {
        return None;
    }
    Some(
        program
            .uses
            .iter()
            .map(|used| used.parts.join(".").to_ascii_lowercase())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli_test::link::LinkContextCache;
    use crate::cli_test::run::run_single_test_capture_prepared;
    use crate::test_support::{create_temp_dir, write_text};

    #[test]
    fn compatible_tests_share_one_in_memory_image_and_run_independently() {
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
        assert!(Arc::ptr_eq(&first_image.image, &second_image.image));

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
    fn tests_with_module_level_declarations_keep_the_individual_path() {
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
            assert!(test.compiled.is_none());
            let (outcome, _) = run_single_test_capture_prepared(&test.path, None, None, None, None);
            assert_eq!(outcome, crate::cli_test::report::TestOutcome::Pass);
        }
    }
}
