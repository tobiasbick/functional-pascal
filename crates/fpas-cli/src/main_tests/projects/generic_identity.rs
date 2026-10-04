//! Generic parameter identities survive independent unit compilation and warm reuse.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::*;

#[test]
fn cli_rejects_generic_callable_containers_without_concrete_context() {
    let cwd = create_temp_dir("generic-callable-container-context");
    let source = cwd.join("main.fpas");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["main.fpas"]);
    write_text(
        &source,
        "program Main;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        begin const Callbacks := [Identity]; discard Callbacks; end program;",
    );
    for command in ["check", "build", "run"] {
        let args = [command.to_owned(), project.to_string_lossy().into_owned()];
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_ne!(exit, 0, "{command} accepted a polymorphic container");
        assert!(
            stderr.contains("F2006") && stderr.contains("Cannot infer"),
            "{stderr}"
        );
        assert!(!stderr.contains("F9001"), "{stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove temporary source");
}

#[test]
fn generic_parameter_identity_links_nested_captures_and_reuses_compilations() {
    let cwd = create_temp_dir("generic-parameter-identity");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public type Box of (T) = record public Value: T; end record;
        public function Identity of (T)(Value: T): T; begin return Value; end function;
        public function Make of (T)(Value: T): function(): Box of (T);
          function Preserve(Captured: T): T; begin return Captured; end function;
          function Inner of (t)(Other: t): t; begin discard Preserve(Value); return Other; end function;
        begin return function(): Box of (T)
          begin discard Inner(1); return Box(Value := Preserve(Value)); end function;
        end function;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Model as Model; uses Std.Console as Console;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        function Apply of (T)(Callback: function(Input: T): T; Value: T): T; begin return Callback(Value); end function;
        begin
          const Text := Model.Make('text'); const Number := Model.Make(41);
          Console.WriteLn(Identity(Text().Value));
          Console.WriteLn(Identity(Number().Value) + 1);
          const Callback: function(Input: integer): integer := Model.Identity;
          Console.WriteLn(Callback(41) + 1);
          Console.WriteLn(Apply(Model.Identity, 'value'));
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "text\n42\n42\nvalue\n", "")
        );
    }
    let args = ["build".to_owned(), project.to_string_lossy().into_owned()];
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!((exit, stderr.as_str()), (0, ""));
    assert!(stdout.contains("Reused program"), "{stdout}");

    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public function Bad of (T)(Value: T): integer;
          function Inner of (T)(Other: T): T; begin return Value; end function;
        begin return Inner(1); end function;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Model as Model; begin discard Model.Bad('text'); end program;",
    );
    for command in ["check", "build", "run"] {
        let args = [command.to_owned(), project.to_string_lossy().into_owned()];
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_ne!(exit, 0, "{command} accepted mismatched generic identities");
        assert!(stderr.contains("F2006"), "{command}: {stderr}");
        assert!(
            stderr.contains("different declarations"),
            "{command}: {stderr}"
        );
        assert!(
            !stderr.contains("F4008") && !stderr.contains("F9001"),
            "{stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}
