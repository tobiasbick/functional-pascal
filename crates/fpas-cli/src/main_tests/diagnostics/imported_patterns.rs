//! Pattern capabilities and record values survive compiled-unit persistence.
//!
//! Documentation: `docs/pascal/language/functions/discard.md`,
//! `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::{create_temp_dir, fs, support, write_text};
use crate::test_support::{write_library_fpasprj, write_program_fpasprj_with_deps};

#[test]
fn imported_routine_result_proofs_survive_pattern_extraction_and_sidecar_reuse() {
    let dir = create_temp_dir("imported-pattern-capabilities");
    write_library_fpasprj(&dir.join("library.fpasprj"), &["CallableLibrary.fpas"]);
    let project = dir.join("consumer.fpasprj");
    write_program_fpasprj_with_deps(
        &project,
        "consumer.fpas",
        &["consumer.fpas"],
        &["library.fpasprj"],
    );
    write_text(
        &dir.join("CallableLibrary.fpas"),
        r#"
unit CallableLibrary;
function Work(): integer; begin return 7; end function;
public function Make(): option of function(): integer;
begin return Some(Work); end function;
end unit;
"#,
    );
    let path = dir.join("consumer.fpas");
    write_text(
        &path,
        r#"
program Consumer;
uses CallableLibrary as CallableLibraryUnit, Std.Console as Console;
begin
  case CallableLibraryUnit.Make() of
    when Some(const F):
      discard F;
      Console.WriteLn(F());
    when None: null;
  end case;
end.
"#,
    );
    for command in ["check", "run", "check", "run"] {
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[command.into(), project.to_string_lossy().into_owned()],
            &dir,
        );
        assert_eq!(exit, 0, "{command}: {stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert_eq!(
            stdout.replace("\r\n", "\n"),
            if command == "run" { "7\n" } else { "" }
        );
        assert!(
            dir.join("CallableLibrary.fpascu").exists(),
            "compiled unit sidecar missing"
        );
    }
    fs::remove_dir_all(dir).expect("remove imported pattern-capability fixture");
}

#[test]
fn cli_preserves_imported_record_fields_in_fresh_and_reused_sidecars() {
    let dir = create_temp_dir("imported-record-patterns");
    write_library_fpasprj(
        &dir.join("library.fpasprj"),
        &["Original.fpas", "Facade.fpas"],
    );
    let project = dir.join("consumer.fpasprj");
    write_program_fpasprj_with_deps(
        &project,
        "consumer.fpas",
        &["consumer.fpas"],
        &["library.fpasprj"],
    );
    write_text(
        &dir.join("Original.fpas"),
        "unit Original;
      public type State = enum Ready = 31; Busy = 9; end enum;
      public type Flags = record public Enabled: boolean := true; public Current: State; end record;
      public const Settings: Flags := Flags(Current := State.Ready);
      end unit;",
    );
    write_text(
        &dir.join("Facade.fpas"),
        "unit Facade; uses Original as OriginalUnit;\n      public const Copy: OriginalUnit.Flags := OriginalUnit.Settings;\n      public const Enabled: boolean := Copy.Enabled;\n      public const Current: OriginalUnit.State := Copy.Current;\n      end unit;",
    );
    write_text(
        &dir.join("consumer.fpas"),
        "program Consumer; uses Original as OriginalUnit, Facade as Values, Std.Console as Console;\n      begin\n        const Defaulted: OriginalUnit.Flags := OriginalUnit.Flags(Current := OriginalUnit.State.Ready);\n        case Some(Defaulted.Enabled) of when Some(Values.Copy.Enabled): null; when Some(false): panic('false'); when None: panic('none'); end case;\n        case Some(OriginalUnit.State.Ready) of when Some(Values.Copy.Current): null; when Some(OriginalUnit.State.Busy): panic('busy'); when None: panic('none'); end case;\n        if not Values.Copy.Enabled then panic('record global'); end if;\n        if Values.Copy.Current <> Values.Current then panic('enum global'); end if;\n        Console.WriteLn('records');\n      end.",
    );
    for command in ["check", "run", "check", "run"] {
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[command.into(), project.to_string_lossy().into_owned()],
            &dir,
        );
        assert_eq!(exit, 0, "{command}: {stderr}");
        assert!(stderr.is_empty(), "{stderr}");
        assert_eq!(
            stdout.replace("\r\n", "\n"),
            if command == "run" { "records\n" } else { "" }
        );
        assert!(dir.join("Original.fpascu").exists());
        assert!(dir.join("Facade.fpascu").exists());
    }
    fs::remove_dir_all(dir).expect("remove fixture");
}
