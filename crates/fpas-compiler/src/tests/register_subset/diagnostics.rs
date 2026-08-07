use super::*;

#[test]
fn division_by_zero_preserves_code_message_and_source_location() {
    let source = "\
program RegisterDivisionError;
begin
  var X: integer := 7 div 0
end.";
    let old = run_old(source).expect_err("stack path should fail");
    let register = run_register(source).expect_err("register path should fail");

    assert_eq!(register.code, old.code);
    assert_eq!(register.message, old.message);
    assert_eq!(register.span.line(), old.span.line());
    assert_eq!(register.span.column(), old.span.column());
}

#[test]
fn explicit_panic_preserves_diagnostic_contract() {
    let source = "\
program RegisterPanic;
begin
  panic('expected failure')
end.";
    let old = run_old(source).expect_err("stack path should fail");
    let register = run_register(source).expect_err("register path should fail");

    assert_eq!(register.code, old.code);
    assert_eq!(register.message, old.message);
    assert_eq!(register.help, old.help);
    assert_eq!(register.span.line(), old.span.line());
    assert_eq!(register.span.column(), old.span.column());
}

#[test]
fn later_phase_constructs_fail_without_exposing_a_backend_flag() {
    let program = parse_ok(
        "\
program RegisterCall;
uses Std.Console;
begin
  Std.Console.WriteLn('later')
end.",
    );
    let error = crate::compile_register_subset(&program)
        .expect_err("imports and intrinsics belong to a later phase");

    assert!(error[0].message.contains("outside the P3"));
}
