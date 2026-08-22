//! Smoke-runs for repository examples that exit on their own.
//!
//! **Do not** batch-run every file under `examples/` (many demos are interactive TUI
//! programs). Add a new `example_*` test below when adding a console example, then run:
//! `cargo test -p fpas-cli example_`
//! or `scripts/run-non-interactive-examples.ps1` / `scripts/run-non-interactive-examples.sh`.

use super::support;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("fpas-cli crate must live two levels below the repository root")
        .to_path_buf()
}

fn run_example(path: &str, program_args: &[&str]) {
    let root = repo_root();
    let mut args = vec![String::from("run"), path.to_owned()];
    if !program_args.is_empty() {
        args.push("--".to_owned());
        args.extend(program_args.iter().map(|arg| (*arg).to_owned()));
    }
    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &root);
    assert_eq!(
        exit_code, 0,
        "example `{path}` failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stderr.is_empty(), "example `{path}` wrote stderr: {stderr}");
}

fn check_example(path: &str) {
    let root = repo_root();
    let (exit_code, _, stderr) =
        support::run_cli_args_and_capture_output(&[String::from("check"), path.to_owned()], &root);
    assert_eq!(
        exit_code, 0,
        "`fpas check {path}` failed\nstderr:\n{stderr}"
    );
    assert!(
        stderr.is_empty(),
        "`fpas check {path}` wrote stderr: {stderr}"
    );
}

macro_rules! example_run_tests {
    ($(($name:ident, $path:expr $(, $($arg:expr),+)?) ),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                run_example($path, &[$($($arg),+)?]);
            }
        )+
    };
}

#[test]
fn example_check_library_deps_mylib() {
    check_example("examples/pascal/library-deps/mylib/mylib.fpasprj");
}

#[test]
fn example_check_monorepo_workspace() {
    check_example("examples/pascal/monorepo/monorepo.fpasworkspace");
}

example_run_tests! {
    (example_hello, "examples/hello.fpas"),
    (example_fibonacci, "examples/fibonacci.fpas"),
    (example_literals_alias_string_index, "examples/pascal/basics/literals_alias_string_index.fpas"),
    (example_while_repeat, "examples/pascal/control-flow/while_repeat_example.fpas"),
    (example_enum_expression, "examples/pascal/enum-data/expression.fpas"),
    (example_enum_shapes, "examples/pascal/enum-data/shapes.fpas"),
    (example_option, "examples/pascal/error-handling/option_example.fpas"),
    (example_panic, "examples/pascal/error-handling/panic_example.fpas"),
    (example_result, "examples/pascal/error-handling/result_example.fpas"),
    (example_for_downto, "examples/pascal/for/downto_example.fpas"),
    (example_for, "examples/pascal/for/for_example.fpas"),
    (example_dict_for_in, "examples/pascal/for-in/dict_for_in_example.fpas"),
    (example_for_in, "examples/pascal/for-in/for_in_example.fpas"),
    (example_mutable_nested_functions, "examples/pascal/functions/mutable_nested_functions.fpas"),
    (example_nested_functions, "examples/pascal/functions/nested_functions.fpas"),
    (example_go_statement, "examples/pascal/concurrency/go_statement_example.fpas"),
    (example_generic_functions, "examples/pascal/generics/generic_functions.fpas"),
    (example_generic_record_methods, "examples/pascal/generics/generic_record_methods.fpas"),
    (example_higher_order_functions, "examples/pascal/higher-order-functions/higher_order_functions.fpas"),
    (example_pattern_exhaustiveness, "examples/pascal/pattern-matching/exhaustiveness.fpas"),
    (example_pattern_guards, "examples/pascal/pattern-matching/guards.fpas"),
    (example_record_counter, "examples/pascal/record-methods/counter.fpas"),
    (example_record_point, "examples/pascal/record-methods/point.fpas"),
    (example_record_rectangle, "examples/pascal/record-methods/rectangle.fpas"),
    (example_record_defaults_update, "examples/pascal/records/defaults_with_update.fpas"),
    (example_array_basics, "examples/pascal/std/array_basics.fpas"),
    (example_args_basics, "examples/pascal/std/args_basics.fpas", "alpha", "beta"),
    (example_console_cells_basics, "examples/pascal/std/console_cells_basics.fpas"),
    (example_dict_basics, "examples/pascal/std/dict_basics.fpas"),
    (example_env_basics, "examples/pascal/std/env_basics.fpas"),
    (example_fs_basics, "examples/pascal/std/fs_basics.fpas"),
    (example_json_basics, "examples/pascal/std/json_basics.fpas"),
    (example_parse_basics, "examples/pascal/std/parse_basics.fpas"),
    (example_path_basics, "examples/pascal/std/path_basics.fpas"),
    (example_proc_basics, "examples/pascal/std/proc_basics.fpas"),
    (example_random_basics, "examples/pascal/std/random_basics.fpas"),
    (example_str_basics, "examples/pascal/std/str_basics.fpas"),
    (example_task_basics, "examples/pascal/std/task_basics.fpas"),
    (example_time_basics, "examples/pascal/std/time_basics.fpas"),
    (example_units_basic, "examples/pascal/units-basic/units-basic.fpasprj"),
    (example_library_deps_app, "examples/pascal/library-deps/app/app.fpasprj"),
    (example_monorepo_hello, "examples/pascal/monorepo/apps/hello/hello.fpasprj"),
}
