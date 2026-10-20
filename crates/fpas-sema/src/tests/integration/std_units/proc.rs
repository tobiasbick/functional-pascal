use super::{check_errors, check_ok};

#[test]
fn proc_run_accepts_string_command_and_string_array_args() {
    check_ok(
        "program T;\nuses Std.Proc as Proc;\nbegin\n  const Status: Result of (integer, Proc.ProcError) := Proc.Run('tool', ['--help']);\n  const Qualified: Result of (integer, Proc.ProcError) := Proc.Run('tool', ['--version']);\nend.",
    );
}

#[test]
fn proc_capture_api_exposes_current_executable_and_process_output() {
    check_ok(
        "program T;\nuses Std.Proc as Proc;\nbegin\n  const Executable: Result of (string, Proc.ProcError) := Proc.CurrentExecutable();\n  const Captured: Result of (Proc.ProcessOutput, Proc.ProcError) :=\n    Proc.RunCapture('tool', ['--version']);\nend.",
    );
}

#[test]
fn proc_run_rejects_non_string_args_array() {
    let errs = check_errors(
        "program T;\nuses Std.Proc as Proc;\nbegin\n    const Status: Result of (integer, Proc.ProcError) := Proc.Run('tool', [1, 2, 3]);\nend.",
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

#[test]
fn proc_run_capture_rejects_non_string_args_array() {
    let errs = check_errors(
        "program T;\nuses Std.Proc as Proc;\nbegin\n    const Captured: Result of (Proc.ProcessOutput, Proc.ProcError) := Proc.RunCapture('tool', [1, 2, 3]);\nend.",
    );

    assert!(
        errs.iter().any(|e| e.message.contains("array of string")),
        "{errs:#?}"
    );
}

#[test]
fn process_options_have_explicit_records_and_cancellable_result_signatures() {
    check_ok(
        r#"program ProcessOptions;
uses Std.Proc as Proc, Std.Tasks as Tasks;
procedure Probe(Token: Tasks.CancellationToken);
begin
  const Environment: Proc.EnvironmentVariable := Proc.EnvironmentVariable(Name := 'KEY', Value := None);
  const Options: Proc.ProcessOptions := Proc.DefaultOptions() with
    WorkingDirectory := Some('.');
    Environment := [Environment];
    Input := 'input';
    TimeoutMillis := Some(100);
    MaxOutputBytes := 20;
    CancellationToken := Some(Token);
  end with;
  const Status: result of (integer, Proc.ProcError) := Proc.RunWithOptions(Options := Options, Args := [], Command := 'tool');
  const Output: result of (Proc.ProcessOutput, Proc.ProcError) := Proc.RunCaptureWithOptions(Command := 'tool', Args := [], Options := Options);
end procedure;
begin end."#,
    );
}

#[test]
fn process_options_require_explicit_defaults_and_typed_option_fields() {
    for expression in [
        "Proc.ProcessOptions()",
        "Proc.EnvironmentVariable(Name := 'KEY')",
        "Proc.DefaultOptions() with TimeoutMillis := 1; end with",
        "Proc.DefaultOptions() with MaxOutputBytes := 1.0; end with",
        "Proc.DefaultOptions() with Environment := ['KEY']; end with",
        "Proc.DefaultOptions() with CancellationToken := Some(1); end with",
    ] {
        assert!(
            !check_errors(&format!(
                "program Invalid; uses Std.Proc as Proc; begin discard {expression}; end."
            ))
            .is_empty(),
            "accepted {expression}"
        );
    }
}
