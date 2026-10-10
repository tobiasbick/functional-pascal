//! Explicit distinct conversions, rejected implicit conversions, and rejected arithmetic.
//!
//! Documentation: `docs/pascal/language/types/distinct-types.md`

use fpas_diagnostics::codes::{SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME, SEMA_WRONG_ARGUMENT_COUNT};

use super::{check_errors, check_ok};

const PRELUDE: &str = "program T;
  uses Std.Console;
  type UserId = distinct integer;
  type OrderId = distinct integer;
  type Raw = integer;
  type Name = distinct string;
  procedure Show(Id: UserId); begin WriteLn(integer(Id)); end procedure;";

fn program(body: &str) -> String {
    format!("{PRELUDE}\nbegin\n{body}\nend.")
}

/// Asserts one diagnostic whose message and help contain the given fragments.
fn assert_rejected(body: &str, code: fpas_diagnostics::DiagnosticCode, message: &str, help: &str) {
    let errors = check_errors(&program(body));
    assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
    let error = &errors[0];
    assert_eq!(error.code, code, "{body}: {errors:#?}");
    assert!(error.message.contains(message), "{body}: {errors:#?}");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|text| text.contains(help)),
        "{body}: {errors:#?}"
    );
}

#[test]
fn explicit_conversions_wrap_unwrap_and_round_trip() {
    check_ok(&program(
        "const Id: UserId := UserId(41);
         const Next: UserId := UserId(integer(Id) + 1);
         const Through: integer := Raw(Next);
         const Same: UserId := UserId(Next);
         const Order: OrderId := OrderId(integer(Id));
         const Text: string := string(Name('Ada'));
         const Size: integer := string(Name('Ada')).Length();
         Show(Next);
         WriteLn(integer(Next), Through, Text, Size);
         const Ids: array of UserId := [Id, Next];
         const Found: Option of UserId := Some(Id);",
    ));
}

#[test]
fn implicit_conversions_in_either_direction_are_rejected() {
    assert_rejected(
        "const Id: UserId := 42;",
        SEMA_TYPE_MISMATCH,
        "expected `UserId`, found `integer`",
        "`UserId(Value)`",
    );
    assert_rejected(
        "const Raw: integer := UserId(1);",
        SEMA_TYPE_MISMATCH,
        "expected `integer`, found `UserId`",
        "`integer(Value)`",
    );
    assert_rejected(
        "Show(1);",
        SEMA_TYPE_MISMATCH,
        "expected `UserId`, found `integer`",
        "`UserId(Value)`",
    );
}

#[test]
fn swapped_domain_arguments_are_rejected() {
    assert_rejected(
        "Show(OrderId(1));",
        SEMA_TYPE_MISMATCH,
        "expected `UserId`, found `OrderId`",
        "`UserId(integer(Value))`",
    );
    assert_rejected(
        "const Id: UserId := UserId(OrderId(1));",
        SEMA_TYPE_MISMATCH,
        "directly to distinct type `UserId`",
        "`UserId(integer(Value))`",
    );
}

#[test]
fn conversions_require_the_exact_underlying_type() {
    assert_rejected(
        "const Id: UserId := UserId(1.5);",
        SEMA_TYPE_MISMATCH,
        "Cannot convert `real` to distinct type `UserId`",
        "underlying type `integer`",
    );
    assert_rejected(
        "const Value: real := real(UserId(1));",
        SEMA_TYPE_MISMATCH,
        "Cannot unwrap distinct type `UserId` to `real`",
        "`integer(Value)`",
    );
}

#[test]
fn builtin_type_name_calls_only_unwrap_distinct_values() {
    assert_rejected(
        "const Value: integer := integer(3.5);",
        SEMA_TYPE_MISMATCH,
        "`integer(...)` only unwraps a distinct type value",
        "`Trunc`",
    );
    assert_rejected(
        "const Value: integer := Raw(3);",
        SEMA_TYPE_MISMATCH,
        "`Raw(...)` only unwraps a distinct type value",
        "`IntToReal`",
    );
}

#[test]
fn conversions_take_exactly_one_positional_value() {
    assert_rejected(
        "const Id: UserId := UserId(1, 2);",
        SEMA_WRONG_ARGUMENT_COUNT,
        "expects 1 argument, got 2",
        "`UserId(Value)`",
    );
    assert_rejected(
        "const Id: UserId := UserId(Value := 1);",
        SEMA_TYPE_MISMATCH,
        "takes one positional value",
        "`UserId(Value)`",
    );
}

#[test]
fn arithmetic_is_not_inherited() {
    for (body, symbol) in [
        ("const Id: UserId := UserId(1) + UserId(2);", "+"),
        ("const Id: integer := UserId(1) * 2;", "*"),
        ("const Id: integer := UserId(4) div 2;", "div"),
        ("const Id: UserId := -UserId(1);", "-"),
        ("const Text: Name := Name('a') + Name('b');", "+"),
    ] {
        let errors = check_errors(&program(body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert!(
            errors[0].message.contains(&format!(
                "Operator `{symbol}` is not defined for distinct type"
            )),
            "{body}: {errors:#?}"
        );
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("use a record or functions")),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn std_output_and_builtin_operations_do_not_unwrap_implicitly() {
    assert_rejected(
        "WriteLn(UserId(1));",
        SEMA_TYPE_MISMATCH,
        "`WriteLn` does not accept distinct type `UserId` implicitly",
        "`integer(Value)`",
    );
    assert_rejected(
        "const Size: integer := Name('Ada').Length();",
        SEMA_UNKNOWN_NAME,
        "has no dot operation `Length`",
        "`string(Value).Length()`",
    );
}

#[test]
fn distinct_conversions_are_values_not_spawnable_calls() {
    assert_rejected(
        "go UserId(1);",
        SEMA_TYPE_MISMATCH,
        "`go` requires a function or procedure call, but `UserId` constructs a value",
        "Spawn a named function",
    );
}
