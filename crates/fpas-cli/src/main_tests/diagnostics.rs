use super::*;

#[test]
fn cli_renders_lex_stage_output() {
    let source = "program LexFail;\nbegin\n  @\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("lex.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "lex.fpas:3:3: error[F0001]: Unexpected character `@`\n  help: Remove this character or replace it with a valid Pascal token such as `:=`, `;`, `(`, or an identifier.\n"
    );
}

#[test]
fn cli_renders_parse_stage_output() {
    let source = "program ParseFail\nbegin\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("parse.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "parse.fpas:2:1: error[F1001]: Expected `;`, found `begin`\n  help: Insert `;` here.\n"
    );
}

#[test]
fn cli_renders_sema_stage_output() {
    let source = "program SemaFail;\nbegin\n  x := 1;\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("sema.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "sema.fpas:3:3: error[F2003]: Undefined identifier `x`\n  help: Check spelling or declare the variable or constant.\n"
    );
}

#[test]
fn cli_renders_compile_stage_output() {
    let diagnostic = Diagnostic::error(
        COMPILE_INTRINSIC_ARITY_MISMATCH,
        DiagnosticStage::Compile,
        "Std.Console.ReadLn takes no arguments",
        Some("Remove all arguments from this call.".to_string()),
        SourceSpan::new(0, 1, 4, 9),
    );

    let rendered = render_cli_diagnostic("compile.fpas", &diagnostic);
    assert_eq!(
        rendered,
        "compile.fpas:4:9: error[F3003]: Std.Console.ReadLn takes no arguments\n  help: Remove all arguments from this call."
    );
}

#[test]
fn cli_renders_runtime_stage_output() {
    let source = "program RuntimeFail;\nbegin\n  panic('boom');\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("runtime.fpas", source);

    assert_eq!(exit_code, 2);
    assert_eq!(
        stderr_output,
        "runtime.fpas:3:3: error[F4010]: panic: boom\n  help: Remove the panic or guard the failing condition before calling panic.\n"
    );
}

#[test]
fn cli_reports_compiler_directive_syntax_as_lex_error() {
    let source = "program Fail;\n{$R+}\nbegin\nend.\n";
    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("directive.fpas", source);

    assert_eq!(exit_code, 1);
    assert!(stdout_output.is_empty());
    assert_eq!(
        stderr_output,
        "directive.fpas:2:1: error[F0010]: `{$...}` is not valid source syntax\n  help: Remove this sequence. Put shared declarations in another `.fpas` file and import the unit with `uses`.\n"
    );
}
