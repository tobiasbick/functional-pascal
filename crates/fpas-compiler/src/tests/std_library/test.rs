//! Compiler integration tests for `Std.Test`.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use super::super::{compile_and_run, compile_ok, compile_run_error};
use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;

#[test]
fn std_test_assert_equals_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(4, 2 + 2);
  AssertTrue(1 + 1 = 2);
  AssertFalse(1 = 2)
end.",
    );
}

#[test]
fn std_test_assert_equals_failure_reports_code() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(4, 5)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 4, got 5"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_assert_true_failure() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertTrue(false)
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected true, got false"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_fail_with_message() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  Fail('boom')
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(err.message.contains("boom"), "message={}", err.message);
}

#[test]
fn std_test_assert_equals_string_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals('hello', 'hel' + 'lo')
end.",
    );
}

#[test]
fn std_test_assert_equals_boolean_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(true, 1 = 1)
end.",
    );
}

#[test]
fn std_test_assert_equals_real_passes() {
    compile_ok(
        "\
program T;
uses Std.Test;
begin
  AssertEquals(1.5, 3.0 / 2.0)
end.",
    );
}

#[test]
fn std_test_assert_equals_string_failure_reports_values() {
    let err = compile_run_error(
        "\
program T;
uses Std.Test;
begin
  AssertEquals('want', 'got')
end.",
    );
    assert_eq!(err.code, RUNTIME_TEST_ASSERTION_FAILED);
    assert!(
        err.message.contains("expected 'want', got 'got'"),
        "message={}",
        err.message
    );
}

#[test]
fn std_test_skip_does_not_fail() {
    compile_and_run(
        "\
program T;
uses Std.Test;
begin
  Skip('later')
end.",
    );
}
