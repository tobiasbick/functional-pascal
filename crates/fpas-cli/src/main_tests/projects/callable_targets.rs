use super::*;

const FACTORY: &str = r#"unit Demo.Factory;

public type Handler = function(Value: integer): integer;

public type Holder = record
  Hidden: Handler;
  public Apply: Handler;
  public Notify: procedure();
end record;

public function Make(Base: integer): Handler;
begin
  return function(Input: integer): integer begin
    return Base + Input;
  end function;
end function;

public function Counter(InitialCount: integer): function(): integer;
function Next(): integer;
begin
  Count := Count + 1;
  return Count;
end function;
begin
   var Count: integer := InitialCount;
  return Next;
end function;

procedure Notify();
begin
  null;
end procedure;

public function Create(): Holder;
begin
  return Holder(Hidden := Make(0), Apply := Make(30), Notify := Notify);
end function;

end unit;
"#;

#[test]
fn imported_returned_and_field_callables_run_through_the_cli() {
    let cwd = create_temp_dir("imported-callable-targets");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("factory.fpas"), FACTORY);
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Factory as Factory; uses Std.Console as Console;
begin
  Console.WriteLn(Factory.Make(3)(5));
  const Functions: array of (Factory.Handler) := [Factory.Make(40)];
  Console.WriteLn(Functions[0](2));
  Console.WriteLn(Factory.Create().Apply(12));
  (Factory.Create().Notify)();
  discard Factory.Make(0)(1);
  const Next: function(): integer := Factory.Counter(40);
  Console.WriteLn((Next)());
  Console.WriteLn((Next)());
end program;"#,
    );
    let (exit_code, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit_code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "8\n42\n42\n41\n42\n");
}

#[test]
fn unused_callable_results_have_located_json_diagnostics() {
    let cwd = create_temp_dir("callable-result-diagnostics");
    let main = cwd.join("main.fpas");
    write_text(
        &main,
        "program Main;\nfunction Value(): integer; begin return 1; end function;\nbegin (Value)(); end program;\n",
    );
    let args = ["check", "--diagnostics", "json", &main.to_string_lossy()].map(str::to_owned);
    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary source");
    assert_eq!(exit_code, 1);
    assert!(stdout.is_empty());
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{stderr}");
    assert_eq!(records[0]["code"], "F2006");
    assert_eq!(records[0]["message"], "Function result must be consumed");
    assert_eq!(records[0]["location"]["start"]["line"], 3);
    assert!(
        records[0]["hint"]
            .as_str()
            .is_some_and(|help| help.contains("discard"))
    );
}

#[test]
fn computed_callable_fields_preserve_imported_visibility() {
    for call in ["Factory.Create().Hidden(1)", "(Factory.Create().Hidden)(1)"] {
        let cwd = create_temp_dir("private-computed-callable-field");
        let project = cwd.join("app.fpasprj");
        support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
        write_text(&cwd.join("factory.fpas"), FACTORY);
        write_text(
            &cwd.join("main.fpas"),
            &format!(
                "program Main;\nuses Demo.Factory as Factory;\nbegin discard {call}; end program;\n"
            ),
        );
        let args =
            ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        fs::remove_dir_all(&cwd).expect("remove temporary project");
        assert_eq!(exit, 1, "{call}: {stderr}");
        assert!(stdout.is_empty());
        let records = stderr
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1, "{call}: {stderr}");
        assert!(
            records[0]["message"]
                .as_str()
                .is_some_and(|message| message.contains("Hidden") && message.contains("private")),
            "{call}: {stderr}"
        );
        assert_eq!(records[0]["location"]["start"]["line"], 3);
    }
}
