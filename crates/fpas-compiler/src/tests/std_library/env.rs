use super::*;

#[test]
fn std_env_reports_missing_variable_as_none() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Env, Std.Option;
begin
  WriteLn(Std.Option.IsNone(Get('__FPAS_ENV_TEST_MISSING_7E5F9A21__')));
  WriteLn(Exists('__FPAS_ENV_TEST_MISSING_7E5F9A21__'))
end.",
    );
    assert_eq!(out.lines, vec!["true", "false"]);
}

#[test]
fn std_env_reads_existing_path_variable() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Env, Std.Option;
begin
  WriteLn(Exists('PATH'));
  WriteLn(Std.Option.IsSome(Get('PATH')))
end.",
    );
    assert_eq!(out.lines, vec!["true", "true"]);
}

#[test]
fn std_env_qualified_get_works() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Env, Std.Option;
begin
  WriteLn(Std.Option.IsSome(Std.Env.Get('PATH')))
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}
