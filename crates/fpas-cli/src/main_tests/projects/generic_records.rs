//! Generic records crossing independently compiled unit interfaces.
//! See `docs/pascal/language/types/generics.md`.

use super::*;

#[test]
fn equatable_private_types_cross_transitive_constructor_abis_and_reused_sidecars() {
    let cwd = create_temp_dir("equatable-private-abi");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    let hidden = cwd.join("src/hidden.fpas");
    let source = r#"unit Data.Hidden;
type Box of T = record Value: T; end record;
type Marker of T: Equatable = error Failed; end error;
function Mark<T: Equatable>(Value: T): Marker of T;
begin return Marker.Failed; end function;
public function Work<T: Equatable>(Value: T): boolean;
begin
  const M := Mark(Box(Value := Value));
  return M is Marker.Failed;
end function;
end unit;"#;
    write_text(&hidden, source);
    write_text(
        &cwd.join("src/facade.fpas"),
        r#"unit Data.Facade;
uses Data.Hidden as Hidden;
public function Run<T: Equatable>(Value: T): boolean;
begin return Hidden.Work(Value); end function;
end unit;"#,
    );
    let main = cwd.join("src/main.fpas");
    let mut program = String::from(
        r#"program T;
uses Data.Facade as Facade;
begin
  if not Facade.Run(1) or not Facade.Run('text') then panic('private ABI'); end if;
end."#,
    );
    let mut sidecars = None;
    for _ in 0..2 {
        // Change only the program to exercise imported metadata from reused units.
        program.push('\n');
        write_text(&main, &program);
        for command in ["check", "run"] {
            let (code, _, stderr) = support::run_cli_args_and_capture_output(
                &[command.into(), project.to_string_lossy().into_owned()],
                &cwd,
            );
            assert_eq!(code, 0, "{command}: {stderr}");
        }
        let current = ["hidden", "facade"].map(|name| {
            let path = cwd.join(format!("src/{name}.fpascu"));
            (
                fs::read(&path).expect("compiled dependency sidecar"),
                fs::metadata(path)
                    .expect("sidecar metadata")
                    .modified()
                    .expect("sidecar modification time"),
            )
        });
        if let Some(previous) = &sidecars {
            assert_eq!(
                &current, previous,
                "unchanged units must reuse their sidecars"
            );
        }
        sidecars = Some(current);
    }
    write_text(
        &hidden,
        &source
            .replace("Value: T; end record", "Value: array of T; end record")
            .replace("Box(Value := Value)", "Box(Value := [Value])"),
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_ne!(code, 0, "unsupported private fields must stay rejected");
    assert!(stderr.contains("Equatable"), "{stderr}");
    assert!(!stderr.contains("FP9001"), "{stderr}");
    fs::remove_dir_all(cwd).expect("remove private ABI fixture");
}

#[test]
fn equatable_records_and_native_errors_cross_reused_sidecars() {
    let cwd = create_temp_dir("equatable-units");
    let project = cwd.join("equal.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    let unit = cwd.join("src/equal.fpas");
    let source = r#"unit Data.Equal;
public type Marker of T: Equatable = error Failed; end error;
public type Box of T = record public Value: T; end record;
public function Same<T: Equatable>(A: T; B: T): boolean; begin return A = B; end function;
public function Mark<T: Equatable>(Value: T): Marker of T; begin return Marker.Failed; end function;
end unit;"#;
    write_text(&unit, source);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program T;
uses Data.Equal as Equal;
type Point = record X: integer; end record;
begin
  const P := Point(X := 5);
  if not Equal.Same(P, Point(X := 5)) then panic('imported equality'); end if;
  const M := Equal.Mark(Equal.Box(Value := P));
  if not (M is Equal.Marker.Failed) then panic('imported error'); end if;
end."#,
    );
    for _ in 0..2 {
        // Rebuild the program while retaining the unchanged dependency sidecar.
        let main = cwd.join("src/main.fpas");
        let text = fs::read_to_string(&main).expect("main source");
        write_text(&main, &(text + "\n"));
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{stderr}");
        assert!(cwd.join("src/equal.fpascu").exists());
    }
    let image = fs::read_dir(&cwd)
        .expect("project artifacts")
        .map(|entry| entry.expect("artifact entry").path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "fpascp")
        })
        .expect("linked program image");
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["run".into(), image.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(code, 0, "{stderr}");
    write_text(
        &unit,
        &source.replace("Same<T: Equatable>", "Same<T: Comparable>"),
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["run".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("Comparable"), "{stderr}");
    fs::remove_dir_all(cwd).expect("remove equality fixture");
}

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
