//! Individually exported declarations preserve unit initialization order.
//!
//! **Documentation:** `docs/pascal/program-structure/visibility.md`

use super::*;

#[test]
fn individual_declarations_export_each_name_and_preserve_initialization_order() {
    let cwd = create_temp_dir("individual-declarations");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/values.fpas"),
        "unit App.Values;
        mutable var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
        public type Box = record public Value: Count; end record;
        public type Count = integer;
        public const A: integer := 10;
        public const B: integer := 20;
        const Hidden: integer := 30;
        public var First: integer := Next();
        public var Second: integer := Next();
        var Third: integer := Next();
        public mutable var Left: integer := 4;
        public mutable var Right: integer := 5;
        public function Total(): integer; begin return Calls; end function;
        end unit;",
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main; uses App.Values, Std.Console;
        begin var Value: Box := record Value := A + B; end;
            WriteLn(Value.Value); WriteLn(First); WriteLn(Second); WriteLn(Total());
            Left := Left + 10; Right := Right + 20;
            WriteLn(Left); WriteLn(Right);
        end.",
    );
    for _ in 0..2 {
        let (code, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(stdout, "30\n1\n2\n3\n14\n25\n");
        assert!(stderr.is_empty(), "{stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
