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
