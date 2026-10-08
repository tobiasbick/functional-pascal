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

#[path = "record_construction/mod.rs"]
mod record_construction;

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, source).expect("fixture source");
}

fn build(root: &Path, main: &Path) -> fpas_build::BuiltProgram {
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
    .expect("program build")
}

fn run(program: fpas_build::BuiltProgram) -> Vec<String> {
    let mut vm = fpas_vm::Vm::new(program.executable);
    vm.run().expect("compiled program execution");
    vm.output().lines.clone()
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
          if Value is Some(Model.Shape.Rect(const W, const H)) and W > 0 then
            return W * H;
          end if;
          return 0;
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
    let source = "program Demo;
        uses Demo.Api as Api, Demo.Types as Model, Other.Helper, Std.Console as Console, Std.Tasks as Jobs;
        begin
          var Counter: integer := 1;
          Api.Increase(var Counter);
          Api.Apply(Api.Increase, var Counter);
          Api.Increase(var Api.Total);
          Api.Items.Push(2);
          const Shape: Model.Shape := Model.Shape.Rect(Height := 3, Width := 4);
          const Point: Model.Point := Api.Make(Value := 10);
          const Bound: function(Value: integer): integer := Point.Add;
          const Increment: function(Value: integer): integer := Api.Next;
          const Pending: task of integer := go Api.Next(Value := 6);
          Console.WriteLn(Api.Next(Value := 9), ' ', Next(9));
          Console.WriteLn(Counter, ' ', Api.Total, ' ', Api.Items.Length(), ' ', Api.Answer);
          Console.WriteLn(Api.Area(Shape), ' ', Point.Add(3), ' ', Bound(4), ' ', Increment(4));
          Console.WriteLn(Point.Status = Model.State.Ready, ' ', Jobs.Wait(Pending));
          Console.WriteLn(Api.StateCode(Some(Model.State.Ready)), ' ', Api.StateCode(Some(Model.State.Busy)), ' ', Api.StateCode(None));
          Console.WriteLn(Api.BoolCode(Some(true)), ' ', Api.BoolCode(Some(false)), ' ', Api.BoolCode(None));
          Console.WriteLn(Api.NestedArea(Some(Shape)), ' ', Api.NestedArea(None));
        end.";
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
    fs::remove_dir_all(&root).ok();
}
