//! Smoke-runs for repository examples that exit on their own.
//!
//! **Do not** batch-run every file under `examples/` (many demos are interactive TUI/graph
//! programs). Extend [`NON_INTERACTIVE_EXAMPLES`] when adding a new console example, then run:
//! `cargo test -p fpas-cli non_interactive_examples_run_successfully`
//! or `scripts/run-non-interactive-examples.ps1` / `scripts/run-non-interactive-examples.sh`.

use super::support;
use std::path::{Path, PathBuf};

struct ExampleCase {
    path: &'static str,
    args: &'static [&'static str],
}

/// Project/workspace paths for `fpas check` smoke runs from the repository root.
const NON_INTERACTIVE_CHECK_EXAMPLES: &[&str] = &[
    "examples/pascal/library-deps/mylib/mylib.fpasprj",
    "examples/pascal/monorepo/monorepo.fpasworkspace",
];

/// Canonical allowlist for automated example runs (CI, agents, local smoke).
const NON_INTERACTIVE_EXAMPLES: &[ExampleCase] = &[
    ExampleCase {
        path: "examples/hello.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/fibonacci.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/basics/literals_alias_string_index.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/control-flow/while_repeat_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/enum-data/expression.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/enum-data/shapes.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/error-handling/option_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/error-handling/panic_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/error-handling/result_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/for/downto_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/for/for_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/for-in/dict_for_in_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/for-in/for_in_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/functions/mutable_nested_functions.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/functions/nested_functions.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/concurrency/go_statement_example.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/generics/generic_functions.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/generics/generic_record_methods.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/higher-order-functions/higher_order_functions.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/pattern-matching/exhaustiveness.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/pattern-matching/guards.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/record-methods/counter.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/record-methods/point.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/record-methods/rectangle.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/records/defaults_with_update.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/array_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/args_basics.fpas",
        args: &["alpha", "beta"],
    },
    ExampleCase {
        path: "examples/pascal/std/dict_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/env_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/fs_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/json_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/parse_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/path_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/proc_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/random_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/str_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/task_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/std/time_basics.fpas",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/units-basic/units-basic.fpasprj",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/library-deps/app/app.fpasprj",
        args: &[],
    },
    ExampleCase {
        path: "examples/pascal/monorepo/apps/hello/hello.fpasprj",
        args: &[],
    },
];

#[test]
fn non_interactive_check_examples_succeed() {
    let root = repo_root();

    for path in NON_INTERACTIVE_CHECK_EXAMPLES {
        let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
            &[String::from("check"), (*path).to_owned()],
            &root,
        );

        assert_eq!(
            exit_code, 0,
            "`fpas check {path}` failed\nstderr:\n{stderr}"
        );
        assert!(
            stderr.is_empty(),
            "`fpas check {path}` wrote stderr: {stderr}"
        );
    }
}

#[test]
fn non_interactive_examples_run_successfully() {
    let root = repo_root();

    for example in NON_INTERACTIVE_EXAMPLES {
        let args = cli_args_for(example);
        let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &root);

        assert_eq!(
            exit_code, 0,
            "example `{}` failed\nstdout:\n{}\nstderr:\n{}",
            example.path, stdout, stderr
        );
        assert!(
            stderr.is_empty(),
            "example `{}` wrote stderr: {stderr}",
            example.path
        );
    }
}

fn cli_args_for(example: &ExampleCase) -> Vec<String> {
    let mut args = vec![example.path.to_owned()];
    if !example.args.is_empty() {
        args.push("--".to_owned());
        args.extend(example.args.iter().map(|arg| (*arg).to_owned()));
    }
    args
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("fpas-cli crate must live two levels below the repository root")
        .to_path_buf()
}
