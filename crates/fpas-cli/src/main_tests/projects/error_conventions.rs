//! Result parser callable values and try propagation across reused unit interfaces.
//! See `docs/pascal/std/text/parse.md`.

use super::*;

#[test]
fn parser_values_and_error_types_survive_independent_unit_reuse() {
    let cwd = create_temp_dir("parser-value-units");
    let project = cwd.join("parsers.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/parsing.fpas"),
        r#"
unit App.Parsing;
uses Std.Text.Parse as Numbers;
public function IntReader(): function(Text: string): Result of (integer, Numbers.ParseError);
begin return Numbers.Int; end function;
public function ReadInt(Text: string): Result of (integer, Numbers.ParseError);
begin return Numbers.Int(Text); end function;
public function Forward(Text: string): Result of (boolean, Numbers.ParseError);
begin const Value := try Numbers.Int(Text); return Ok(Value > 0); end function;
end unit;
"#,
    );
    let main = cwd.join("src/main.fpas");
    let source = r#"
program T;
uses App.Parsing as Readers, Std.Console as Console;
begin
  const Read := Readers.IntReader();
  if Read(Text := '42').Unwrap() <> 42 then panic('parser value'); end if;
  if Readers.ReadInt(Text := 'invalid') is Ok(_) then panic('returned error'); end if;
  if Readers.Forward('1').Unwrap() <> true then panic('try success'); end if;
  case Readers.Forward('invalid') of
    when Ok(_): panic('try must propagate');
    when Error(const Message):
      if not ('invalid' in Message.Message()) then panic('error payload'); end if;
  end case;
  Console.WriteLn('parsers verified');
end.
"#;
    let sidecar = cwd.join("src/parsing.fpascu");
    let mut compiled_unit = None;
    for consumer_change in ["", "\n// Rebuild the consumer while reusing the unit.\n"] {
        write_text(&main, &format!("{source}{consumer_change}"));
        let (code, stdout, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(stdout.trim(), "parsers verified");
        let bytes = fs::read(&sidecar).expect("independently compiled parser unit");
        if let Some(previous) = &compiled_unit {
            assert_eq!(&bytes, previous, "unit retained on consumer rebuild");
        }
        compiled_unit = Some(bytes);
    }
    write_text(
        &main,
        "program T; uses App.Parsing as Readers; function Wrong(): Result of (boolean, integer); begin discard try Readers.ReadInt('1'); return Ok(true); end function; begin end.",
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("FP3006") && stderr.contains("try"),
        "{stderr}"
    );
    assert!(!stderr.contains("FP9001"), "{stderr}");
    fs::remove_dir_all(cwd).expect("remove fixture");
}
