//! Recursive forward types survive separate unit compilation and sidecar reuse.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::*;

#[test]
fn run_cli_imports_forward_recursive_types_and_reuses_compiled_units() {
    let cwd = create_temp_dir("run-forward-recursive-types");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(&cwd.join("src/tree.fpas"), "unit App.Tree;
      const Initial: State := State.Ready;
      public function Create(): Node; begin return Node.Empty(); end function;
      public type Node = record
        public Children: array of Edge;
        public Status: State;
        public Seed: integer := 7;
        public static function Empty(): Node; begin return record Children := []; Status := Initial; end; end function;
        public function IsReady(Self: Node): boolean; begin return Self.Status = State.Ready; end function;
      end record;
      public type Edge = enum Stop; More(Next: Option of Node); end enum;
      public type State = enum Ready; Busy; end enum;
      public type Alias = Node;
      end unit;");
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main; uses App.Tree, Std.Console;
      begin var Root: Alias := Create();
        var Child: Edge := Edge.More(Some(Root));
        var Populated: Node := Root with Children := [Child]; end with;
        var Bound: function(): boolean := Populated.IsReady;
        WriteLn(Bound()); WriteLn(Populated.Seed);
      end.",
    );
    for _ in 0..2 {
        let (code, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(stdout, "true\n7\n");
        assert!(stderr.is_empty(), "{stderr}");
    }
    assert!(cwd.join("src/tree.fpascu").exists());
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
