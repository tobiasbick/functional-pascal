//! Named and `var` calls to routines imported from compiled units.
//!
//! Documentation: `docs/pascal/language/functions/parameters.md`,
//! `docs/pascal/language/functions/var-parameters.md`

#![allow(
    clippy::expect_used,
    reason = "compiled-unit fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};

use fpas_build::{BuildOptions, ProgramArtifactTarget, build_program_artifact};
use fpas_project::{build_unit_graph_for_program, load_project};

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
fn named_calls_bind_parameters_and_variant_fields_of_imported_compiled_units() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("fpas-named-arguments-{}", std::process::id()));
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
        &root.join("src/math.fpas"),
        "unit Demo.Math;
         public type Shape = enum Rect(Width: integer; Height: integer); end enum;
         public function Sub(Left: integer; Right: integer): integer;
         begin return Left - Right; end function;
         public function Area(S: Shape): integer;
         begin case S of when Shape.Rect(W, H): return W * H; end case; end function;\nend unit;",
    );
    write(
        &main,
        "program Demo;
         uses Demo.Math, Std.Console;
         begin Std.Console.WriteLn(Sub(Right := 1, Left := 10)); end.",
    );

    let cold = build(&root, &main);
    assert_eq!(run(cold), ["9"]);

    write(
        &main,
        "program Demo;
         uses Demo.Math, Std.Console;
         begin
           Std.Console.WriteLn(Demo.Math.Sub(Right := 2, Left := 10));
           Std.Console.WriteLn(Area(Shape.Rect(Height := 3, Width := 4)));
         end.",
    );
    let warm = build(&root, &main);
    assert_eq!(warm.counters().compiled, 0, "the unit sidecar is reused");
    assert_eq!(run(warm), ["8", "12"]);
    fs::remove_dir_all(&root).ok();
}

#[test]
fn var_parameters_and_their_function_types_cross_compiled_units() {
    let root: PathBuf =
        std::env::temp_dir().join(format!("fpas-var-parameters-{}", std::process::id()));
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
        &root.join("src/counters.fpas"),
        "unit Demo.Counters;
         public type Step = procedure(var Value: integer);
         public var Total: integer := 0;
         public procedure Increase(var Value: integer);
         begin Value := Value + 1; end procedure;
         public procedure Apply(Action: Step; var Value: integer);
         begin Action(var Value); end procedure;
end unit;",
    );
    write(
        &main,
        "program Demo;
         uses Demo.Counters, Std.Console;
         begin
           var Counter: integer := 1;
           Increase(var Counter);
           Apply(Increase, var Counter);
           Increase(var Demo.Counters.Total);
           Std.Console.WriteLn(Counter, ' ', Total);
         end.",
    );
    assert_eq!(run(build(&root, &main)), ["3 1"]);
    let warm = build(&root, &main);
    assert_eq!(warm.counters().compiled, 0, "the unit sidecar is reused");
    assert_eq!(run(warm), ["3 1"]);
    fs::remove_dir_all(&root).ok();
}

#[test]
fn mutating_intrinsics_write_imported_arrays_and_forward_across_compiled_units() {
    let root =
        std::env::temp_dir().join(format!("fpas-mutating-intrinsics-{}", std::process::id()));
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
        &root.join("src/state.fpas"),
        r#"unit Demo.State;

public type Holder = record public Items: array of integer; end record;
public var Items: array of integer := [1];
public var State: Holder := Holder( Items := [2] );
public procedure Append(var Target: array of integer; Value: integer);
begin Target.Push(Value); end procedure;
end unit;"#,
    );
    write(
        &main,
        r#"program Demo;
uses Demo.State, Std.Console;
begin
  Demo.State.Items.Push(3);
  Demo.State.State.Items.Push(4);
  Append(var Demo.State.Items, 5);
  Append(var State.Items, 6);
  WriteLn(Demo.State.Items.Pop(), ' ', Demo.State.Items.Pop(), ' ', Demo.State.Items[0]);
  WriteLn(State.Items.Pop(), ' ', Demo.State.State.Items.Pop(), ' ', State.Items[0]);
end."#,
    );
    assert_eq!(run(build(&root, &main)), ["5 3 1", "6 4 2"]);
    let warm = build(&root, &main);
    assert_eq!(warm.counters().compiled, 0, "the unit sidecar is reused");
    assert_eq!(run(warm), ["5 3 1", "6 4 2"]);
    fs::remove_dir_all(&root).ok();
}
