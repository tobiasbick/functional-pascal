//! Runs the repository FPAS regression suite under `tests/` as one Cargo test per theme.
//!
//! Filter: `cargo test -p fpas-cli fpas_suite_`
//! Full one-shot outside Cargo: `fpas test tests/` or `fpas test tests/suite.fpasprj`.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("fpas-cli crate must live two levels below the repository root")
        .to_path_buf()
}

fn run_fpas_suite(rel_dir: &str) {
    let root = repo_root();
    let (exit, _, stderr) = super::support::run_cli_args_and_capture_output(
        &[String::from("test"), rel_dir.to_owned()],
        &root,
    );
    assert_eq!(exit, 0, "fpas test {rel_dir} failed\nstderr:\n{stderr}");
}

macro_rules! fpas_suite_tests {
    ($(($name:ident, $rel_dir:expr)),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                run_fpas_suite($rel_dir);
            }
        )+
    };
}

fpas_suite_tests! {
    (fpas_suite_stdlib_array, "tests/stdlib/array/"),
    (fpas_suite_stdlib_closures, "tests/stdlib/closures/"),
    (fpas_suite_stdlib_console, "tests/stdlib/console/"),
    (fpas_suite_stdlib_conv, "tests/stdlib/conv/"),
    (fpas_suite_stdlib_dict, "tests/stdlib/dict/"),
    (fpas_suite_stdlib_fs, "tests/stdlib/fs/"),
    (fpas_suite_stdlib_json, "tests/stdlib/json/"),
    (fpas_suite_stdlib_math, "tests/stdlib/math/"),
    (fpas_suite_stdlib_option, "tests/stdlib/option/"),
    (fpas_suite_stdlib_random, "tests/stdlib/random/"),
    (fpas_suite_stdlib_result, "tests/stdlib/result/"),
    (fpas_suite_stdlib_str, "tests/stdlib/str/"),
    (fpas_suite_stdlib_toml, "tests/stdlib/toml/"),
    (fpas_suite_stdlib_tui2, "tests/stdlib/tui2/"),
    (fpas_suite_concurrency, "tests/concurrency/"),
    (fpas_suite_console, "tests/console/"),
    (fpas_suite_graph, "tests/graph/"),
}

#[test]
fn fpas_suite_runner() {
    let root = repo_root();
    let (exit, _, stderr) = super::support::run_cli_args_and_capture_output(
        &[String::from("test"), String::from("tests/runner/")],
        &root,
    );
    assert_eq!(exit, 0, "fpas test tests/runner/ failed\nstderr:\n{stderr}");
    assert!(
        stderr.contains("SKIP  skip_test.fpas"),
        "expected skip_test.fpas to be reported as skipped\nstderr:\n{stderr}"
    );
}
