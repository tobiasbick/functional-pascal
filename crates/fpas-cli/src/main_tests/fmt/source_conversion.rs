//! Migrated stage-three sources retain meaning through CLI formatting and execution.
//!
//! Documentation: `docs/pascal/tools/fmt-style.md` and
//! `docs/pascal/program-structure/units.md`.

use super::*;
use crate::test_support::write_program_fpasprj;

#[test]
fn formatting_preserves_scopes_branch_ownership_comments_and_evaluation() {
    let cwd = create_temp_dir("fmt-source-meaning");
    let source_path = cwd.join("main.fpas");
    let source = r#"program Meaning;
uses Std.Console as Console; // explicit alias
uses Std.Bits as Bits;
 var Trace: integer := 0;
function Next(Digit: integer): integer;
begin Trace := Trace * 10 + Digit; return Digit; end function;
function Check(Digit: integer): boolean;
begin return Next(Digit) = 2; end function;
function Apply(F: function(Value: integer): integer; Value: integer): integer;
begin return F(Value); end function;
begin
  const Label: string := 'outer';
  if Check(1) then panic('wrong outer branch');
  else // nested conditional keeps its own closer
    if Check(2) then
      const Label: string := 'inner'; Console.WriteLn(Label);
    else panic('wrong nested branch'); end if;
  end if;
  begin // explicit block keeps its local binding
    const Label: string := 'block'; Console.WriteLn(Label);
  end;
  case 2 of
    when 1: panic('wrong arm');
    when 2: const Label: string := 'arm'; Console.WriteLn(Label);
    else panic('wrong fallback');
  end case;
  Console.WriteLn(Label);
  Console.WriteLn(Apply(function(Value: integer): integer
    begin return Bits.BitOr(Value, Next(4)); end function, Next(3)));
  const Skipped: boolean := false and Check(5);
  const Eager: boolean := Check(6) xor Check(7);
  Console.WriteLn(Trace);
  Console.WriteLn(not 3 < 2 and 2 + 3 * 4 = 14);
  // old syntax in a comment: end. uses Std.Str; 1 shl 2
  Console.WriteLn('end. // uses Std.Str; 1 shl 2');
end program; // final comment
"#;
    write_text(&source_path, source);
    let path = source_path.to_string_lossy().into_owned();
    let before = run_cli_args_and_capture_output(&["run".into(), path.clone()], &cwd);
    assert_eq!(before.0, 0, "{}", before.2);
    assert_eq!(
        before.1,
        "inner\nblock\narm\nouter\n7\n123467\ntrue\nend. // uses Std.Str; 1 shl 2\n"
    );

    let formatted = run_cli_args_and_capture_output(&["fmt".into(), path.clone()], &cwd);
    assert_eq!(formatted.0, 0, "{}", formatted.2);
    let text = fs::read_to_string(&source_path).expect("formatted source");
    assert_ne!(text, source);
    for comment in [
        "// explicit alias",
        "// nested conditional keeps its own closer",
        "// explicit block keeps its local binding",
        "// old syntax in a comment: end. uses Std.Str; 1 shl 2",
        "// final comment",
    ] {
        assert_eq!(text.matches(comment).count(), 1, "{text}");
    }
    let after = run_cli_args_and_capture_output(&["run".into(), path.clone()], &cwd);
    assert_eq!(after.0, 0, "{}", after.2);
    assert_eq!(after.1, before.1);
    let checked = run_cli_args_and_capture_output(&["fmt".into(), "--check".into(), path], &cwd);
    assert_eq!(checked.0, 0, "{}", checked.2);
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn formatting_project_preserves_resolved_owners_with_identical_export_names() {
    let cwd = create_temp_dir("fmt-resolved-owners");
    let project = cwd.join("app.fpasprj");
    write_program_fpasprj(&project, "src/main.fpas", &["src/*.fpas"]);
    for (unit, result) in [("Left", 11), ("Right", 22)] {
        write_text(
            &cwd.join(format!("src/{unit}.fpas")),
            &format!(
                "unit App.{unit}; // owner {unit}\n public type Choice = enum Selected; end enum; public function GetNumber(): integer; begin return {result}; end function; end unit;"
            ),
        );
    }
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main; uses App.Left as First; // keep owner\n uses App.Right as Second; uses Std.Console as Console; function GetNumber(): integer; begin return 33; end function; begin case fIrSt.Choice.Selected of when First.Choice.Selected: Console.WriteLn(First.GetNumber()); end case; case Second.Choice.Selected of when Second.Choice.Selected: Console.WriteLn(Second.GetNumber()); end case; Console.WriteLn(GetNumber()); end program;",
    );
    let path = project.to_string_lossy().into_owned();
    let before = run_cli_args_and_capture_output(&["run".into(), path.clone()], &cwd);
    assert_eq!(before.0, 0, "{}", before.2);
    assert_eq!(before.1, "11\n22\n33\n");
    let formatted = run_cli_args_and_capture_output(&["fmt".into(), path.clone()], &cwd);
    assert_eq!(formatted.0, 0, "{}", formatted.2);
    let after = run_cli_args_and_capture_output(&["run".into(), path.clone()], &cwd);
    assert_eq!(after.0, 0, "{}", after.2);
    assert_eq!(after.1, before.1);
    let checked = run_cli_args_and_capture_output(&["fmt".into(), "--check".into(), path], &cwd);
    assert_eq!(checked.0, 0, "{}", checked.2);
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn formatting_rejects_ambiguous_and_obsolete_syntax_without_rewriting_source() {
    let cwd = create_temp_dir("fmt-reject-conversion");
    let path = cwd.join("main.fpas");
    for (source, hint) in [
        (
            r#"program P; begin const X: boolean := true and false or true; end program;"#,
            "Mixed logical",
        ),
        (
            r#"program P; begin const X: boolean := 1 < Next() < 3; end program;"#,
            "Chained comparison",
        ),
        (
            "program P; begin if true then null; else if false then null; end if; end program;",
            "elsif",
        ),
        (
            "program P; uses Std.Str; begin null; end program;",
            "explicit alias",
        ),
        ("program P; begin null; end.", "end program"),
        (
            r#"program P; begin const X: integer := 1 shl 2; end program;"#,
            "Std.Bits",
        ),
        (
            "program P; begin if true then null else null; end if; end program;",
            "Every statement ends",
        ),
        ("program P; begin null;; end program;", "empty statements"),
    ] {
        write_text(&path, source);
        let (exit, stdout, stderr) = run_cli_args_and_capture_output(
            &["fmt".into(), path.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(exit, 1, "{source}: {stderr}");
        assert!(stdout.is_empty(), "{source}: {stdout}");
        assert!(stderr.contains(hint), "{source}: {stderr}");
        assert_eq!(fs::read_to_string(&path).expect("unchanged source"), source);
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn semantic_check_rejects_unresolved_ownership_and_alias_collisions() {
    let cwd = create_temp_dir("fmt-reject-ownership");
    let path = cwd.join("main.fpas");
    for (body, hint) in [
        ("var S: string := Trim(' a ');", "Text."),
        ("var S: string := Std.Str.Trim(' a ');", "Text.Trim"),
        (r#"begin const TEXT: integer := 1; end;"#, "import alias"),
        (
            "if true then var Local: integer := 1; else Local := 2; end if;",
            "Local",
        ),
    ] {
        let source = format!("program P; uses Std.Str as Text; begin {body} end program;");
        write_text(&path, &source);
        let (exit, _, stderr) = run_cli_args_and_capture_output(
            &["check".into(), path.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_ne!(exit, 0, "{source}");
        assert!(stderr.contains(hint), "{source}: {stderr}");
        assert_eq!(fs::read_to_string(&path).expect("unchanged source"), source);
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
