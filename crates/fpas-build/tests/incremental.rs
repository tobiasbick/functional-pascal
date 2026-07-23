#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "incremental filesystem fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_build::{BuildOptions, build_program};
use fpas_project::{build_unit_graph, load_project, resolve_program_units};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-incremental-build-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, source).expect("fixture source");
}

fn base_source(private_body: &str, offset: i64) -> String {
    format!(
        "unit Demo.Base;
         const Offset: integer := {offset};
         private function Hidden(Value: integer): integer;
         begin {private_body} end;
         function AddOffset(Value: integer): integer;
         begin return Value + Offset end;"
    )
}

struct Fixture {
    root: PathBuf,
    manifest: PathBuf,
    base: PathBuf,
}

impl Fixture {
    fn create() -> Self {
        let root = temp_dir();
        let manifest = root.join("demo.fpasprj");
        write(
            &manifest,
            r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
        );
        let base = root.join("src/base.fpas");
        write(&base, &base_source("return Value", 1));
        write(
            &root.join("src/consumer.fpas"),
            "unit Demo.Consumer;
             uses Demo.Base;
             function Run(): integer;
             begin return AddOffset(41) end;",
        );
        write(
            &root.join("src/main.fpas"),
            "program Demo;
             uses Demo.Consumer, Std.Console;
             begin Std.Console.WriteLn(Run()) end.",
        );
        Self {
            root,
            manifest,
            base,
        }
    }

    fn build(&self) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
        let project = load_project(&self.manifest).expect("project loading");
        let graph =
            build_unit_graph(&project.source_files, &project.link_meta).expect("unit graph");
        let main = project.main.expect("program main");
        let source = fs::read_to_string(main).expect("main source");
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let selection = resolve_program_units(&graph, &program.uses).expect("reachable units");
        build_program(&graph, &selection, &program, &BuildOptions::default())
    }
}

#[test]
fn cold_warm_and_interface_invalidation_rebuild_the_minimum_units() {
    let fixture = Fixture::create();

    let cold = fixture.build().expect("cold build");
    assert_eq!(cold.counters().compiled, 2);
    assert_eq!(cold.counters().sidecar_reused, 0);
    let mut vm = fpas_vm::Vm::new(cold.chunk);
    vm.run().expect("cold linked execution");
    assert_eq!(vm.output().lines, ["42"]);

    let warm = fixture.build().expect("warm build");
    assert_eq!(warm.counters().compiled, 0);
    assert_eq!(warm.counters().parsed, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    let warm_project = load_project(&fixture.manifest).expect("warm project loading");
    let warm_graph = build_unit_graph(&warm_project.source_files, &warm_project.link_meta)
        .expect("warm unit graph");
    assert!(
        warm_graph.iter().all(|(_, node)| !node.has_parsed_source()),
        "valid sidecars must build the dependency graph without parsing source ASTs"
    );

    write(&fixture.base, &base_source("return Value + 100", 1));
    let private_change = fixture.build().expect("private implementation rebuild");
    assert_eq!(private_change.counters().compiled, 1);
    assert_eq!(private_change.counters().sidecar_reused, 1);

    write(&fixture.base, &base_source("return Value + 100", 2));
    let public_change = fixture.build().expect("public interface rebuild");
    assert_eq!(public_change.counters().compiled, 2);
    assert_eq!(public_change.counters().sidecar_reused, 0);

    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn failed_rebuild_preserves_previous_valid_sidecar() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let sidecar = fixture.base.with_extension("fpascu");
    let previous = fs::read(&sidecar).expect("initial sidecar");

    write(
        &fixture.base,
        "unit Demo.Base;
         function AddOffset(Value: integer): integer;
         begin return 'wrong' end;",
    );
    assert!(fixture.build().is_err());
    assert_eq!(
        fs::read(&sidecar).expect("preserved sidecar"),
        previous,
        "failed compilation must not replace the last valid object"
    );

    fs::remove_dir_all(&fixture.root).ok();
}
