//! Source aliases across canonical compiled-unit interfaces and cached artifacts.
//!
//! Documentation: `docs/pascal/program-structure/units.md`

#![allow(
    clippy::expect_used,
    reason = "compiled-unit fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};

use fpas_build::{BuildOptions, ProgramArtifactTarget, build_program_artifact};
use fpas_project::{build_unit_graph_for_program, load_project};

#[path = "distinct_types/mod.rs"]
mod distinct_types;
#[path = "record_construction/mod.rs"]
mod record_construction;

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, source).expect("fixture source");
}

fn build(root: &Path, main: &Path) -> fpas_build::BuiltProgram {
    try_build(root, main).expect("program build")
}

fn try_build(root: &Path, main: &Path) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
    let project = load_project(&root.join("demo.fpasprj")).expect("project loading");
    let graph = build_unit_graph_for_program(main, &project.source_files, &project.link_meta)
        .expect("program unit graph");
    let source_paths = graph
        .source_paths()
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .expect("fixture-relative source")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    build_program_artifact(
        &graph,
        ProgramArtifactTarget {
            path: &root.join("demo.fpascp"),
            source: &fs::read(main).expect("main source"),
            source_paths: &source_paths,
        },
        &BuildOptions::default(),
    )
}

fn run(program: fpas_build::BuiltProgram) -> Vec<String> {
    let mut vm = fpas_vm::Vm::new(program.executable);
    vm.run().expect("compiled program execution");
    vm.output().lines.clone()
}

#[test]
fn ancestor_enum_members_and_child_unit_constants_keep_their_own_aliases() {
    let root = std::env::temp_dir().join(format!("fpas-owner-aliases-{}", std::process::id()));
    let main = root.join("src/main.fpas");
    write(
        &root.join("demo.fpasprj"),
        "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/**/*.fpas']\n",
    );
    write(
        &root.join("src/parent.fpas"),
        "unit A; public type B = enum C = 7; D = 8; end enum; public function IsC(Value: B): boolean; begin return Value is B.C; end function; end unit;",
    );
    write(
        &root.join("src/child.fpas"),
        "unit A.B; public const C: integer := 42; end unit;",
    );
    write(
        &main,
        "program Demo; uses A as Parent, A.B as Child; begin if not Parent.IsC(Parent.B.C) or Child.C <> 42 then panic('owning alias changed a constant'); end if; end.",
    );
    run(build(&root, &main));
    run(build(&root, &main));
    fs::remove_dir_all(root).ok();
}

#[test]
fn aliases_preserve_types_calls_globals_and_reuse_canonical_unit_artifacts() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("fpas-import-aliases-{}", std::process::id()));
    let main = root.join("src/main.fpas");
    write(
        &root.join("demo.fpasprj"),
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &root.join("src/types.fpas"),
        "unit Demo.Types;
        public type State = enum Ready; Busy; end enum;
        public type Point = record
          public X: integer;
          public Status: State;
          public Offset: integer := 2;
          public static function Create(Value: integer): Point;
          begin return Point( X := Value, Status := State.Ready ); end function;
          public function Add(Self: Point; Value: integer): integer;
          begin return Self.X + Value; end function;
        end record;
        public type Shape = enum Rect(Width: integer; Height: integer); end enum;
        public const ReadyChoice: State := State.Ready;
        public const BusyChoice: State := State.Busy;
        public const Yes: boolean := true;
        public const No: boolean := false;
        public type Step = procedure(var Value: integer);
        end unit;",
    );
    write(
        &root.join("src/api.fpas"),
        "unit Demo.Api;
        uses Demo.Types as Model;
        public const Answer: integer := 42;
        public var Total: integer := 0;
        public var Items: array of integer := [1];
        public function Next(Value: integer): integer;
        begin return Value + 1; end function;
        public procedure Increase(var Value: integer);
        begin Value := Value + 1; end procedure;
        public procedure Apply(Action: Model.Step; var Value: integer);
        begin Action(var Value); end procedure;
        public function Area(Value: Model.Shape): integer;
        begin case Value of when Model.Shape.Rect(const W, const H): return W * H; end case; end function;
        public function StateCode(Value: option of Model.State): integer;
        begin
          case Value of
            when Some(Model.ReadyChoice): return 1;
            when Some(Model.BusyChoice): return 2;
            when None: return 0;
          end case;
        end function;
        public function BoolCode(Value: option of boolean): integer;
        begin
          case Value of
            when Some(Model.Yes): return 1;
            when Some(Model.No): return 2;
            when None: return 0;
          end case;
        end function;
        public function NestedArea(Value: option of Model.Shape): integer;
        begin
          return case Value of
            when Some(Model.Shape.Rect(const W, const H)) if W > 0: W * H;
            when Some(_), None: 0;
          end case;
        end function;
        public function Make(Value: integer): Model.Point;
        begin return Model.Point.Create(Value); end function;
        end unit;",
    );
    write(
        &root.join("src/other.fpas"),
        "unit Other.Helper;
        public function Next(Value: integer): integer;
        begin return Value * 2; end function;
        end unit;",
    );
    let source = "program Demo;\n        uses Demo.Api as Api, Demo.Types as Model, Other.Helper as Helper, Std.Console as Console, Std.Tasks as Jobs;\n        begin scope\n          var Counter: integer := 1;\n          Api.Increase(var Counter);\n          Api.Apply(Api.Increase, var Counter);\n          Api.Increase(var Api.Total);\n          Api.Items.PushInPlace(2);\n          const Shape: Model.Shape := Model.Shape.Rect(Height := 3, Width := 4);\n          const Point: Model.Point := Api.Make(Value := 10);\n          const Bound: function(Value: integer): integer := Point.Add;\n          const Increment: function(Value: integer): integer := Api.Next;\n          const Pending: task of integer := go Api.Next(Value := 6);\n          Console.WriteLn(Api.Next(Value := 9), ' ', Helper.Next(9));\n          Console.WriteLn(Counter, ' ', Api.Total, ' ', Api.Items.Length(), ' ', Api.Answer);\n          Console.WriteLn(Api.Area(Shape), ' ', Point.Add(3), ' ', Bound(4), ' ', Increment(4));\n          Console.WriteLn(Point.Status = Model.State.Ready, ' ', Jobs.Wait(Pending));\n          Console.WriteLn(Api.StateCode(Some(Model.State.Ready)), ' ', Api.StateCode(Some(Model.State.Busy)), ' ', Api.StateCode(None));\n          Console.WriteLn(Api.BoolCode(Some(true)), ' ', Api.BoolCode(Some(false)), ' ', Api.BoolCode(None));\n          Console.WriteLn(Api.NestedArea(Some(Shape)), ' ', Api.NestedArea(None));\n         end scope; end.";
    write(&main, source);
    let cold = build(&root, &main);
    assert_eq!(cold.counters().compiled, 3);
    let expected = [
        "10 18",
        "3 1 2 42",
        "12 13 14 5",
        "true 7",
        "1 2 0",
        "1 2 0",
        "12 0",
    ];
    assert_eq!(run(cold), expected);
    write(
        &main,
        &source
            .replace("as Api", "as Service")
            .replace("Api.", "Service."),
    );
    let warm = build(&root, &main);
    assert_eq!(
        warm.counters().compiled,
        0,
        "source alias renaming preserves unit sidecars"
    );
    assert_eq!(run(warm), expected);
    for (before, after, repair) in [
        ("Model.State.Ready", "Model.Ready", "Model.state.ready"),
        ("Model.Shape.Rect", "Rect", "Model.shape.Rect"),
        ("Api.Next(Value := 9)", "Next(Value := 9)", "Service.Next"),
    ] {
        let invalid = source
            .replace(before, after)
            .replace("as Api", "as Service")
            .replace("Api.", "Service.");
        write(&main, &invalid);
        let error = try_build(&root, &main)
            .err()
            .expect("cached interfaces keep required qualifiers");
        assert!(
            error.diagnostics().iter().any(|record| {
                record.diagnostic.code == fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME
                    && record.diagnostic.help.as_ref().is_some_and(|help| {
                        help.to_ascii_lowercase()
                            .contains(&repair.to_ascii_lowercase())
                    })
            }),
            "{error}"
        );
    }
    fs::remove_dir_all(&root).ok();
}
