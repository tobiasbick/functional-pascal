//! AP42 search results and ordering constraints through independently compiled units.

use super::*;

#[test]
fn collection_contract_survives_cold_and_reused_unit_interfaces() {
    let cwd = create_temp_dir("collection-unit-interfaces");
    let project = cwd.join("app.fpasprj");
    let main = cwd.join("src/main.fpas");
    let unit = cwd.join("src/collections.fpas");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &unit,
        r#"
unit App.Collections;
public type Phase = enum Last; First; Middle; end enum;
public function Sorted<T: Comparable>(Items: array of T): array of T;
begin return Items.Sort(); end function;
public function Positions(): array of Option of integer;
begin
  return [
    [1].IndexOf(1),
    [3, 1].FindIndex(function(X: integer): boolean begin return X = 1; end function),
    'é😀'.IndexOf('😀'),
    'é😀'.LastIndexOf('x')
  ];
end function;
end unit;
"#,
    );
    let source = r#"
program Main;
uses App.Collections as Collections, Std.Console as Console;
begin
  const Positions := Collections.Positions();
  if ((Positions[0] <> Some(0)) or (Positions[1] <> Some(1))) or
     ((Positions[2] <> Some(1)) or (Positions[3] <> None)) then
    panic('imported search results');
  end if;
  const Phases := Collections.Sorted([Collections.Phase.Middle, Collections.Phase.Last]);
  if Phases[0] <> Collections.Phase.Last then panic('imported enum sorting'); end if;
  Console.WriteLn('collections verified');
end.
"#;
    write_text(&main, source);
    let cold = support::run_cli_and_capture_output(&project, &cwd);
    assert_eq!(cold.0, 0, "{}", cold.2);
    assert_eq!(cold.1, "collections verified\n");
    let compiled_unit = fs::read(unit.with_extension("fpascu")).expect("compiled collection unit");

    // Rebuild the consumer from a reused unit interface, not only the whole program image.
    write_text(
        &main,
        &format!("{source}\n// Recheck the consumer with cached unit declarations.\n"),
    );
    let reused = support::run_cli_and_capture_output(&project, &cwd);
    assert_eq!(reused.0, 0, "{}", reused.2);
    assert_eq!(reused.1, "collections verified\n");
    assert_eq!(
        fs::read(unit.with_extension("fpascu")).expect("reused collection unit"),
        compiled_unit
    );
    fs::remove_dir_all(cwd).expect("remove collection project");
}
