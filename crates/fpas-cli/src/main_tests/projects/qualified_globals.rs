//! Source-unit aliases must preserve global reads, writes, and initialization.

use super::*;

#[test]
fn aliases_preserve_global_value_paths_and_closure_access() {
    let cwd = create_temp_dir("run-alias-globals");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/state.fpas"),
        r#"unit App.State;

public type Model = record
  public Values: array of (integer);
end record;

public mutable var Count: integer := 2;
public mutable var Grid: array of (array of (integer)) := [[1, 2], [3, 4]];
public mutable var Data: Model := Model(Values := [5, 6]);

public function GetCount(): integer;
begin
  return Count;
end function;

end unit;
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.State as Store;
uses Std.Console as Console;
        begin
          Store.Count := Store.Count + 1;
          Store.Grid[1][0] := Store.Count;
          Store.Data.Values[0] := Store.Grid[1][0] + 4;
          var Reader: function(): integer := function(): integer
            begin return sToRe.GetCount() + Store.Data.Values[0]; end function;
          Console.WriteLn(Reader());
          Console.WriteLn(Store.Grid[0][1]);
        end program;"#,
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("remove test project");
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "10\n2\n");
}
