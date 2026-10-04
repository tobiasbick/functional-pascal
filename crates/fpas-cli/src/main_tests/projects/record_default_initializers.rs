//! Checked default initializers retain private code and nominal identity across units.

use super::*;

#[test]
fn imported_accessors_cannot_hide_effects_in_pure_evaluation() {
    let cwd = create_temp_dir("pure-imported-accessors");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        var State: integer := 0;
        public type Handler = procedure();
        var Active: Option of (Handler) := Option.None;
        public type Box = record
          Value: integer;
        end record;
        function ReadCount(Receiver: Box): integer;
        begin State := State + 1; return Receiver.Value; end function;
        procedure WriteCount(Receiver: Box; Value: integer); begin State := Value; end procedure;
        function ReadAction(Receiver: Box): Option of (Handler); begin return Active; end function;
        procedure WriteAction(Receiver: Box; Value: Option of (Handler)); begin Active := Value; end procedure;
        public function BoxCount(Receiver: Box): integer; begin return ReadCount(Receiver); end function;
        public procedure BoxSetCount(Receiver: Box; Value: integer); begin WriteCount(Receiver, Value); end procedure;
        public function BoxAction(Receiver: Box): Option of (Handler); begin return ReadAction(Receiver); end function;
        public procedure BoxSetAction(Receiver: Box; Value: Option of (Handler)); begin WriteAction(Receiver, Value); end procedure;
        public function NewBox(): Box; begin return Box(Value := 42); end function;
        public function Reads(): integer; begin return State; end function;
        public procedure Fire(Value: Box); begin
          case BoxAction(Value) of
            when Option.Some(const Action): Action();
            when Option.None: null;
          end case;
        end procedure;
        end unit;",
    );
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;
        public type Alias = Model.Box; end unit;",
    );
    let main = cwd.join("main.fpas");
    let prefix = "program Main; uses Demo.Model as Model; uses Demo.Facade as Api;
        uses Std.Console as Console; const Instance: Api.Alias := Model.NewBox();";
    write_text(&main, &format!("{prefix}
        type Settings = record
          Action: function(): integer := function(): integer begin return Model.BoxCount(Instance); end function;
        end record;
        begin
          Console.WriteLn(Model.Reads());
          const Value := Settings();
          Console.WriteLn(Model.Reads());
          Console.WriteLn(Value.Action()); Console.WriteLn(Model.Reads());
          Console.WriteLn(Model.BoxCount((Instance))); Console.WriteLn(Model.Reads());
          Model.BoxSetCount(Instance, 10); Console.WriteLn(Model.Reads());
          Model.BoxSetAction(Instance, Option.Some(procedure() begin Console.WriteLn('handler'); end procedure));
          Model.Fire(Instance);
          Model.BoxSetAction(Instance, Option.None);
          Model.Fire(Instance);
        end program;"));
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "0\n0\n42\n1\n42\n2\n10\nhandler\n", "")
        );
    }
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    for declaration in [
        "type Bad = record Count: integer := Model.BoxCount(Instance); end record;",
        "type Bad = record Count: integer := Model.BoxCount((Instance)); end record;",
        "pure function Evaluate(): integer; begin return Model.BoxCount(Instance); end function;",
        "type Bad = record Action: Option of (Model.Handler) := Model.BoxAction(Instance); end record;",
    ] {
        write_text(
            &main,
            &format!("{prefix} {declaration} begin null; end program;"),
        );
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{stderr}");
        assert!(stdout.is_empty());
        assert!(
            stderr.contains("F2006") && stderr.contains("Pure evaluation cannot call"),
            "{stderr}"
        );
        assert!(!stderr.contains("F9001"), "{stderr}");
    }
    for body in [
        "discard Model.ReadCount(Instance);",
        "Model.WriteCount(Instance, 7);",
        "discard Model.ReadAction(Instance);",
        "Model.WriteAction(Instance, Option.None);",
        "discard Instance.Value;",
    ] {
        write_text(&main, &format!("{prefix} begin {body} end program;"));
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{stderr}");
        assert!(stderr.to_ascii_lowercase().contains("private"), "{stderr}");
        assert!(!stderr.contains("F9001"), "{stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn generic_default_capabilities_survive_aliases_and_only_apply_to_omitted_fields() {
    let cwd = create_temp_dir("generic-default-capability");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("model.fpas"), "unit Demo.Model;
        pure function Empty of (T)(): array of (T); begin return []; end function;
        public type Box of (T) = record public Factory: function(): array of (T) := Empty; end record;
        end unit;");
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;
        public type Data = Model.Box of (integer);
        public type Handles = Model.Box of (channel of (integer)); end unit;",
    );
    let main = cwd.join("main.fpas");
    write_text(&main, "program Main; uses Demo.Facade as Api;
        begin
          const Data := Api.Data();
          const Handles := Api.Handles(Factory := function(): array of (channel of (integer)) begin return []; end function);
          for Item: integer in Data.Factory() do panic('empty'); end for;
          for Item: channel of (integer) in Handles.Factory() do panic('empty'); end for;
        end program;");
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "", ""));
    }
    write_text(
        &main,
        "program Main; uses Demo.Facade as Api; begin const Invalid := Api.Handles(); discard Invalid; end program;",
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("F2006") && stderr.contains("requires resource-free type arguments"),
        "{stderr}"
    );
    assert!(!stderr.contains("F9001"), "{stderr}");
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn aggregate_and_callable_defaults_survive_aliases_and_warm_unit_reuse() {
    let cwd = create_temp_dir("record-default-initializers");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("model.fpas"), "unit Demo.Model;
        const Base: integer := 40;
        pure function Initial(): array of (integer); begin return [Base, 2]; end function;
        public type Settings = record
          public Values: array of (integer) := Initial();
          public Callback: function(Value: integer): integer := function(Value: integer): integer begin return Base + Value; end function;
          public Action: Option of (procedure()) := Option.None;
        end record;
        public type Box of (T) = record public Values: array of (T) := []; end record;
        end unit;");
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;
        public type Settings = Model.Settings;
        public type SettingsList = array of (Settings);
        public type Handles = Model.Box of (channel of (integer));
        end unit;",
    );
    write_text(&cwd.join("main.fpas"), "program Main; uses Demo.Facade as Api; uses Std.Console as Console;
        begin
          var Value := Api.Settings();
          Console.WriteLn(Value.Values[0] + Value.Values[1]);
          Console.WriteLn(Value.Callback(2));
          Value.Values[0] := 99;
          const Other: Api.SettingsList := [Api.Settings()];
          Console.WriteLn(Other[0].Values[0]);
          const Empty := Api.Handles();
          for Item: channel of (integer) in Empty.Values do panic('expected empty'); end for;
          case Value.Action of when Option.None: null; when Option.Some(const Handler): panic('expected absent'); end case;
        end program;");
    let mut artifacts = None;
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "42\n42\n40\n", "")
        );
        let bytes = [
            fs::read(cwd.join("model.fpascu")).unwrap(),
            fs::read(cwd.join("facade.fpascu")).unwrap(),
        ];
        if let Some(previous) = &artifacts {
            assert_eq!(previous, &bytes);
        }
        artifacts = Some(bytes);
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
