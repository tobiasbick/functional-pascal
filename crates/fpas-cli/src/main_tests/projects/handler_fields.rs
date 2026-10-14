//! Project-linking regression tests for optional record handler fields.
//!
//! **Documentation:** `docs/pascal/language/functions/first-class.md`

use super::*;

#[test]
fn run_cli_assigns_and_calls_optional_handler_field() {
    let cwd = create_temp_dir("run-record-handler-field");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Widget as Widget, Std.Console as Console, Std.Conv as Conv;\nprocedure Handle(Value: integer);\nbegin\n  Console.WriteLn(Conv.IntToStr(Value));\nend procedure;\nbegin\n  var B: Widget.Button := Widget.Button.Make(14);\n  const Empty: Widget.Button := Widget.Button(Id := 0);\n  if (Empty.OnClick is Some(_)) then panic('imported handler default'); end if;\n  B.OnClick := Some(Handle);\n  if B.OnClick is Some(_) then\n    B.Click(); end if;\n  B.OnClick := None;\n  B.Click();\nend.",
    );
    write_text(
        &cwd.join("src/widget.fpas"),
        "unit App.Widget;
public type
  Button = record
    public Id: integer;
    public OnClick: Option of procedure(Value: integer) := (None);
    public procedure Click(Self: Button);
    begin
      case Self.OnClick of
        when Some(const Handler):
          Handler(Self.Id);
        when None: null;
      end case;
    end procedure;
    public static function Make(Id: integer): Button;
    begin
      return Button( Id := Id );
    end function;
  end record;
end unit;\n",
    );

    let mut saved_sidecar = None;
    for pass in 0..2 {
        let (exit_code, stdout_output, stderr_output) =
            support::run_cli_and_capture_output(&project_file, &cwd);
        assert_eq!(exit_code, 0, "pass {pass}: {stderr_output}");
        assert_eq!(stdout_output, "14\n");
        assert!(stderr_output.is_empty());
        let sidecar = fs::read(cwd.join("src/widget.fpascu")).expect("compiled widget sidecar");
        if let Some(saved) = &saved_sidecar {
            assert_eq!(
                &sidecar, saved,
                "unchanged dependency sidecar must be reused"
            );
        }
        saved_sidecar = Some(sidecar);
        if pass == 0 {
            // Rebuild the consumer while retaining the persisted dependency interface.
            let main = cwd.join("src/main.fpas");
            let source = fs::read_to_string(&main).expect("consumer source");
            write_text(&main, &format!("{source}\n// Recheck imported defaults.\n"));
        }
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}
