use super::*;

#[test]
fn std_args_reports_count_and_values() {
    let out = compile_and_run_with_args(
        "\
program T;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamCount());
  WriteLn(ParamStr(0));
  WriteLn(Std.Args.ParamStr(1))
end.",
        &["alpha", "beta"],
    );
    assert_eq!(out.lines, vec!["2", "alpha", "beta"]);
}

#[test]
fn std_args_reports_zero_when_no_program_args() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamCount())
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn std_args_param_str_out_of_bounds_reports_runtime_error() {
    let err = compile_run_error_with_args(
        "\
program T;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamStr(1))
end.",
        &["only"],
    );
    assert!(
        err.message.contains("Program argument index 1"),
        "message={}",
        err.message
    );
}

#[test]
fn std_args_param_str_negative_index_reports_runtime_error() {
    let err = compile_run_error_with_args(
        "\
program T;
uses Std.Console, Std.Args;
begin
  WriteLn(ParamStr(-1))
end.",
        &["only"],
    );
    assert!(
        err.message.contains("Negative program argument index"),
        "message={}",
        err.message
    );
}
