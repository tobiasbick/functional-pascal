//! Negative and special-case tests for FPAS files under `tests/`.

use std::path::PathBuf;

use super::support;

fn repo_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("fpas-cli crate must live two levels below the repository root")
        .to_path_buf()
}

fn run_file_expect_failure(rel_path: &str, stderr_contains: Option<&str>) {
    let root = repo_root();
    let path = root.join(rel_path);
    let (exit_code, _stdout, stderr) = support::run_cli_args_and_capture_output(
        &[String::from("run"), path.to_string_lossy().to_string()],
        &root,
    );
    assert_ne!(
        exit_code, 0,
        "expected failure for `{rel_path}`\nstderr:\n{stderr}"
    );
    if let Some(needle) = stderr_contains {
        assert!(
            stderr.contains(needle),
            "stderr for `{rel_path}` should contain `{needle}`\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn std_args_receives_program_arguments_after_cli_separator() {
    let root = repo_root();
    let path = root.join(
        "tests/stdlib/args/std_args_receives_program_arguments_after_cli_separator_cli_args.fpas",
    );
    let args = vec![
        String::from("run"),
        path.to_string_lossy().to_string(),
        String::from("--"),
        String::from("one"),
        String::from("-two"),
    ];
    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &root);
    assert!(stderr.is_empty(), "stderr: {stderr}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "2\none\n-two\n");
}

#[test]
fn concat_rejects_incompatible_element_types() {
    run_file_expect_failure(
        "tests/stdlib/array/concat_rejects_incompatible_element_types_compile_error.fpas",
        Some("right array element"),
    );
}

#[test]
fn array_index_out_of_bounds_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/array/array_index_out_of_bounds_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn flat_map_rejects_scalar_mapper_result_is_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/array/flat_map_rejects_scalar_mapper_result_compile_error.fpas",
        Some("error[F2006]: `Std.Array.FlatMap` mapper must return an array"),
    );
}

#[test]
fn pop_empty_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/array/pop_empty_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn slice_out_of_bounds_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/array/slice_out_of_bounds_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn writeln_without_uses_is_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/console/writeln_without_uses_is_error_compile_error.fpas",
        Some("error[F2003]: Unknown procedure `WriteLn`"),
    );
}

#[test]
fn str_to_int_invalid_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_int_invalid_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn str_to_int_empty_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_int_empty_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn str_to_real_invalid_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_real_invalid_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn str_to_real_empty_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_real_empty_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn str_to_bool_invalid_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_bool_invalid_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn str_to_bool_empty_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/str_to_bool_empty_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn hex_to_int_invalid_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/hex_to_int_invalid_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn hex_to_int_empty_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/conv/hex_to_int_empty_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn dict_index_missing_key_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/dict/dict_index_missing_key_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn arcsin_out_of_range() {
    run_file_expect_failure(
        "tests/stdlib/math/arcsin_out_of_range_runtime_error.fpas",
        None,
    );
}

#[test]
fn arccos_out_of_range() {
    run_file_expect_failure(
        "tests/stdlib/math/arccos_out_of_range_runtime_error.fpas",
        None,
    );
}

#[test]
fn clamp_mixed_types_is_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/math/clamp_mixed_types_is_compile_error_compile_error.fpas",
        Some("same numeric kind"),
    );
}

#[test]
fn min_mixed_types_is_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/math/min_mixed_types_is_compile_error_compile_error.fpas",
        Some("same numeric kind"),
    );
}

#[test]
fn max_mixed_types_is_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/math/max_mixed_types_is_compile_error_compile_error.fpas",
        Some("same numeric kind"),
    );
}

#[test]
fn clamp_reversed_bounds_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/math/clamp_reversed_bounds_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn log_zero_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/math/log_zero_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn log_negative_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/math/log_negative_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn log10_non_positive_error() {
    run_file_expect_failure(
        "tests/stdlib/math/log10_non_positive_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn log2_non_positive_error() {
    run_file_expect_failure(
        "tests/stdlib/math/log2_non_positive_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn sqrt_negative_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/math/sqrt_negative_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn unwrap_none_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/option/unwrap_none_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn unwrap_error_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/result/unwrap_error_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn char_at_out_of_bounds() {
    run_file_expect_failure(
        "tests/stdlib/str/char_at_out_of_bounds_runtime_error.fpas",
        None,
    );
}

#[test]
fn char_at_negative_index() {
    run_file_expect_failure(
        "tests/stdlib/str/char_at_negative_index_runtime_error.fpas",
        None,
    );
}

#[test]
fn delete_out_of_bounds() {
    run_file_expect_failure(
        "tests/stdlib/str/delete_out_of_bounds_runtime_error.fpas",
        None,
    );
}

#[test]
fn insert_out_of_bounds() {
    run_file_expect_failure(
        "tests/stdlib/str/insert_out_of_bounds_runtime_error.fpas",
        None,
    );
}

#[test]
fn set_char_at_out_of_bounds_reports_index_and_length() {
    run_file_expect_failure(
        "tests/stdlib/str/set_char_at_out_of_bounds_runtime_error.fpas",
        Some("error[F4021]: SetCharAt index 10 out of range (length 2)"),
    );
}

#[test]
fn chr_invalid_codepoint() {
    run_file_expect_failure(
        "tests/stdlib/str/chr_invalid_codepoint_runtime_error.fpas",
        None,
    );
}

#[test]
fn chr_rejects_oversized_codepoint() {
    run_file_expect_failure(
        "tests/stdlib/str/chr_rejects_oversized_codepoint_runtime_error.fpas",
        None,
    );
}

#[test]
fn pad_left_negative_width_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/str/pad_left_negative_width_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn pad_right_negative_width_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/str/pad_right_negative_width_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn pad_center_negative_width_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/str/pad_center_negative_width_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn result_and_option_unqualified_unwrap_is_ambiguous_compile_error() {
    run_file_expect_failure(
        "tests/stdlib/result/result_and_option_unqualified_unwrap_is_ambiguous_compile_error.fpas",
        Some("error[F2004]: Ambiguous imported symbol `Unwrap`"),
    );
}

#[test]
fn split_empty_delimiter_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/str/split_empty_delimiter_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn substring_out_of_bounds_is_runtime_error() {
    run_file_expect_failure(
        "tests/stdlib/str/substring_out_of_bounds_is_runtime_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn random_int_reversed_bounds_error() {
    run_file_expect_failure(
        "tests/stdlib/random/random_int_reversed_bounds_error_runtime_error.fpas",
        None,
    );
}

#[test]
fn std_math_no_longer_imports_random_helpers() {
    run_file_expect_failure(
        "tests/stdlib/random/std_math_no_longer_imports_random_helpers_compile_error.fpas",
        Some("Std.Random"),
    );
}
