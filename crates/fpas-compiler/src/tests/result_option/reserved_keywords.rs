/// Tests that `Result`, `Option`, `Ok`, `Error`, `Some`, `None`, `try`, `panic` are
/// reserved keywords and cannot be used as identifiers.
///
/// **Spec:** `docs/pascal/07-error-handling.md`, line 155.
use super::parse_fails;

// ── Variable names ──────────────────────────────────────────────────────

#[test]
fn result_as_variable_name_is_parse_error() {
    parse_fails("program T; var Result: integer := 1; begin end.");
}

#[test]
fn option_as_variable_name_is_parse_error() {
    parse_fails("program T; var Option: integer := 1; begin end.");
}

#[test]
fn ok_as_variable_name_is_parse_error() {
    parse_fails("program T; var Ok: integer := 1; begin end.");
}

#[test]
fn error_as_variable_name_is_parse_error() {
    parse_fails("program T; var Error: integer := 1; begin end.");
}

#[test]
fn some_as_variable_name_is_parse_error() {
    parse_fails("program T; var Some: integer := 1; begin end.");
}

#[test]
fn none_as_variable_name_is_parse_error() {
    parse_fails("program T; var None: integer := 1; begin end.");
}

#[test]
fn try_as_variable_name_is_parse_error() {
    parse_fails("program T; var Try: integer := 1; begin end.");
}

#[test]
fn panic_as_variable_name_is_parse_error() {
    parse_fails("program T; var Panic: integer := 1; begin end.");
}

// ── Function names ──────────────────────────────────────────────────────

#[test]
fn result_as_function_name_is_parse_error() {
    parse_fails(
        "program T;
function Result(): integer;
begin
  return 1
end;
begin
end.",
    );
}

#[test]
fn result_as_local_variable_name_in_method_is_parse_error() {
    parse_fails(
        "program T;
type Holder = record
    Value: integer;
    function Wrap(Self: Holder): integer;
    begin
        var Result: integer := Self.Value;
        return Result
    end;
end;
begin
end.",
    );
}

#[test]
fn ok_as_function_name_is_parse_error() {
    parse_fails(
        "program T;
function Ok(): integer;
begin
  return 1
end;
begin
end.",
    );
}

#[test]
fn try_as_function_name_is_parse_error() {
    parse_fails(
        "program T;
function Try(): integer;
begin
  return 1
end;
begin
end.",
    );
}

#[test]
fn panic_as_function_name_is_parse_error() {
    parse_fails(
        "program T;
function Panic(): integer;
begin
  return 1
end;
begin
end.",
    );
}

// ── Case insensitivity ─────────────────────────────────────────────────

#[test]
fn result_lowercase_as_variable_name_is_parse_error() {
    parse_fails("program T; var result: integer := 1; begin end.");
}

#[test]
fn option_uppercase_as_variable_name_is_parse_error() {
    parse_fails("program T; var OPTION: integer := 1; begin end.");
}

#[test]
fn try_mixed_case_as_variable_name_is_parse_error() {
    parse_fails("program T; var tRy: integer := 1; begin end.");
}

#[test]
fn none_uppercase_as_variable_name_is_parse_error() {
    parse_fails("program T; var NONE: integer := 1; begin end.");
}
