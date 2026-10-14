//! Generic records crossing independently compiled unit interfaces.
//! See `docs/pascal/language/types/generics.md`.

use super::*;

#[test]
fn generic_records_keep_arguments_methods_defaults_and_aliases_across_sidecars() {
    let cwd = create_temp_dir("generic-record-units");
    let project = cwd.join("records.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/boxes.fpas"),
        r#"
unit Data.Boxes;
type T = string;
public type Box of T = record
  public Value: T;
  public function Get(Self: Box of T): T; begin return Self.Value; end function;
end record;
public type Optional of T = record public Value: Option of T := None; end record;
public type Node of T = record public Value: T; public Next: Option of Node of T := None; end record;
public type PrivateBox of T = record Value: T; end record;
public function Make<T>(Value: T): Box of T; begin return Box(Value := Value); end function;
end unit;
"#,
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        r#"
unit Data.Facade;
uses Data.Boxes as Boxes;
public type IntBox = Boxes.Box of integer;
public function Empty(): Boxes.Optional of string; begin return Boxes.Optional(); end function;
public function Nested(): Boxes.Box of Boxes.Box of integer; begin return Boxes.Box(Value := Boxes.Make(9)); end function;
end unit;
"#,
    );
    let main = cwd.join("src/main.fpas");
    write_text(
        &main,
        r#"
program T;
uses Data.Boxes as Boxes, Data.Facade as Facade;
begin
  var I: Facade.IntBox := Facade.IntBox(Value := 7);
  I.Value := 8;
  const S: Boxes.Box of string := Boxes.Make('eight');
  const N: Boxes.Box of Boxes.Box of integer := Facade.Nested();
  const O: Boxes.Optional of integer := Boxes.Optional();
  const E: Boxes.Optional of string := Facade.Empty();
  const Root: Boxes.Node of integer := Boxes.Node(Value := 2, Next := Some(Boxes.Node(Value := 1)));
  if (I.Get() <> 8) or (S.Get() <> 'eight') or (N.Value.Get() <> 9) or (O.Value is Some(_)) or (E.Value is Some(_)) then panic('generic unit records'); end if;
  if Root.Next.Unwrap().Value <> 1 then panic('recursive generic unit record'); end if;
end.
"#,
    );
    for _ in 0..2 {
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{stderr}");
        assert!(
            cwd.join("src/boxes.fpascu").exists(),
            "independent unit sidecar"
        );
    }
    for (statement, message) in [
        (
            "const V: Boxes.PrivateBox of integer := Boxes.PrivateBox(Value := 1);",
            "private",
        ),
        (
            "const V: Boxes.Box of string := Boxes.Make(1);",
            "type mismatch",
        ),
    ] {
        write_text(
            &main,
            &format!("program T; uses Data.Boxes as Boxes; begin {statement} end."),
        );
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["check".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 1, "{stderr}");
        assert!(stderr.to_ascii_lowercase().contains(message), "{stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove project fixture");
}
