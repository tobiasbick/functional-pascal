use super::*;

mod json;
mod unused_results;

#[test]
fn cli_renders_lex_stage_output() {
    let source = "program LexFail;\nbegin\n  @\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("lex.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "lex.fpas:3:3: error[FP1001]: Unexpected character `@`\n  help: Remove this character or replace it with a valid Pascal token such as `:=`, `;`, `(`, or an identifier.\n"
    );
}

#[test]
fn cli_renders_parse_stage_output() {
    let source = "program ParseFail\nbegin\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("parse.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "parse.fpas:2:1: error[FP2001]: Expected `;`, found `begin`\n  help: Insert `;` here.\n"
    );
}

#[test]
fn check_rejects_case_without_an_arm_and_accepts_case_with_an_arm() {
    let cwd = create_temp_dir("check-case-arm");
    let invalid = cwd.join("missing_arm.fpas");
    write_text(
        &invalid,
        "program T; begin case 1 of else return; end case; end.",
    );
    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            invalid.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    assert_ne!(exit_code, 0, "case without an arm was accepted");
    assert!(
        stderr.contains("Expected at least one case arm"),
        "{stderr}"
    );

    let valid = cwd.join("with_arm.fpas");
    write_text(
        &valid,
        "program T; begin case 1 of when 1: return; else return; end case; end.",
    );
    let (exit_code, _, stderr) = support::run_cli_args_and_capture_output(
        &[String::from("check"), valid.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(exit_code, 0, "{stderr}");
}

#[test]
fn cli_renders_sema_stage_output() {
    let source = "program SemaFail;\nbegin\n  x := 1;\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("sema.fpas", source);

    assert_eq!(exit_code, 1);
    assert_eq!(
        stderr_output,
        "sema.fpas:3:3: error[FP3003]: Undefined identifier `x`\n  help: Check spelling or declare the variable or constant.\n"
    );
}

#[test]
fn cli_renders_compile_stage_output() {
    let diagnostic = Diagnostic::error(
        COMPILE_INTRINSIC_ARITY_MISMATCH,
        "Std.Console.ReadLn takes no arguments",
        Some("Remove all arguments from this call.".to_string()),
        SourceSpan::new(0, 1, 4, 9),
    );

    let rendered = render_cli_diagnostic("compile.fpas", &diagnostic);
    assert_eq!(
        rendered,
        "compile.fpas:4:9: error[FP4003]: Std.Console.ReadLn takes no arguments\n  help: Remove all arguments from this call."
    );
}

#[test]
fn cli_renders_runtime_stage_output() {
    let source = "program RuntimeFail;\nbegin\n  panic('boom');\nend.\n";
    let (exit_code, stderr_output) = support::run_and_capture_stderr("runtime.fpas", source);

    assert_eq!(exit_code, 2);
    assert_eq!(
        stderr_output,
        "runtime.fpas:3:3: error[FP5010]: panic: boom\n  help: Remove the panic or guard the failing condition before calling panic.\n"
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
        "directive.fpas:2:1: error[FP1010]: `{$...}` is not valid source syntax\n  help: Remove this sequence. Put shared declarations in another `.fpas` file and import the unit with `uses`.\n"
    );
}

#[test]
fn cli_reports_invalid_comment_form_with_the_valid_syntax() {
    let source = "program Fail;\n{ not a comment }\nbegin\nend.\n";
    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("comment.fpas", source);

    assert_eq!(exit_code, 1);
    assert!(stdout_output.is_empty());
    assert_eq!(
        stderr_output,
        "comment.fpas:2:1: error[FP1013]: `{...}` is not valid comment syntax\n  help: Use `// comment`. For multiple lines, prefix each line with `//`.\n"
    );
}
