#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "incremental filesystem fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_build::{BuildOptions, build_library_units, build_program};
use fpas_project::{build_unit_graph, load_project, resolve_library_units, resolve_program_units};
use fpas_unit::{Digest, decode, encode};

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
         public const Offset: integer := {offset};
         function Hidden(Value: integer): integer;
         begin {private_body} end;
         public function AddOffset(Value: integer): integer;
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
             public function Run(): integer;
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
        self.build_with_options(&BuildOptions::default())
    }

    fn build_with_options(
        &self,
        options: &BuildOptions,
    ) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
        let project = load_project(&self.manifest).expect("project loading");
        let graph =
            build_unit_graph(&project.source_files, &project.link_meta).expect("unit graph");
        let main = project.main.expect("program main");
        let source = fs::read_to_string(main).expect("main source");
        let (program, diagnostics) = fpas_parser::parse(&source);
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
        let selection = resolve_program_units(&graph, &program.uses).expect("reachable units");
        build_program(&graph, &selection, &program, options)
    }

    fn build_library_with_options(
        &self,
        options: &BuildOptions,
    ) -> Result<fpas_build::BuiltUnits, fpas_build::BuildError> {
        let project = load_project(&self.manifest).expect("project loading");
        let graph =
            build_unit_graph(&project.source_files, &project.link_meta).expect("unit graph");
        let selection = resolve_library_units(&graph).expect("library units");
        build_library_units(&graph, &selection, options)
    }
}

fn assert_output(program: fpas_build::BuiltProgram, expected: &str) {
    let mut vm = fpas_vm::Vm::new(program.chunk);
    vm.run().expect("linked execution");
    assert_eq!(vm.output().lines, [expected]);
}

#[test]
fn cold_warm_and_interface_invalidation_rebuild_the_minimum_units() {
    let fixture = Fixture::create();

    let cold = fixture.build().expect("cold build");
    assert_eq!(cold.counters().compiled, 2);
    assert_eq!(cold.counters().sidecar_reused, 0);
    assert_output(cold, "42");

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
    assert_eq!(warm.counters().relinked, 1);
    assert_output(warm, "42");

    write(&fixture.base, &base_source("return Value + 100", 1));
    let private_change = fixture.build().expect("private implementation rebuild");
    assert_eq!(private_change.counters().compiled, 1);
    assert_eq!(private_change.counters().sidecar_reused, 1);
    assert_output(private_change, "42");

    write(&fixture.base, &base_source("return Value + 100", 2));
    let public_change = fixture.build().expect("public interface rebuild");
    assert_eq!(public_change.counters().compiled, 2);
    assert_eq!(public_change.counters().sidecar_reused, 0);
    assert_output(public_change, "43");

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

#[test]
fn corrupt_payload_rebuilds_and_replaces_the_sidecar() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let sidecar = fixture.base.with_extension("fpascu");
    let mut compiled =
        decode(&fs::read(&sidecar).expect("initial sidecar")).expect("sidecar format");
    compiled.object = b"invalid relocatable object".to_vec();
    compiled.identity.object_hash = Digest::of(&compiled.object);
    fs::write(
        &sidecar,
        encode(&compiled).expect("corrupt payload fixture"),
    )
    .expect("corrupt sidecar");

    let rebuilt = fixture.build().expect("payload recovery build");
    assert_eq!(rebuilt.counters().compiled, 1);
    assert_eq!(rebuilt.counters().sidecar_reused, 1);
    assert_output(rebuilt, "42");

    let warm = fixture.build().expect("recovered warm build");
    assert_eq!(warm.counters().compiled, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);

    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn changed_build_options_rebuild_existing_sidecars() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");

    let mut changed_compiler = BuildOptions::default();
    changed_compiler.compiler_version.push_str("-different");
    let compiler_rebuild = fixture
        .build_with_options(&changed_compiler)
        .expect("compiler identity rebuild");
    assert_eq!(compiler_rebuild.counters().compiled, 2);
    assert_eq!(compiler_rebuild.counters().sidecar_reused, 0);

    let mut changed_bytecode = changed_compiler.clone();
    changed_bytecode.bytecode_version += 1;
    let bytecode_rebuild = fixture
        .build_with_options(&changed_bytecode)
        .expect("bytecode identity rebuild");
    assert_eq!(bytecode_rebuild.counters().compiled, 2);
    assert_eq!(bytecode_rebuild.counters().sidecar_reused, 0);

    let mut changed_options = changed_bytecode;
    changed_options.options_hash = Digest::of(b"different compilation options");
    let options_rebuild = fixture
        .build_with_options(&changed_options)
        .expect("option identity rebuild");
    assert_eq!(options_rebuild.counters().compiled, 2);
    assert_eq!(options_rebuild.counters().sidecar_reused, 0);

    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn library_build_reuses_all_units_without_linking_a_program() {
    let fixture = Fixture::create();

    let cold = fixture
        .build_library_with_options(&BuildOptions::default())
        .expect("cold library build");
    assert_eq!(cold.counters().compiled, 2);
    assert_eq!(cold.counters().relinked, 0);

    let warm = fixture
        .build_library_with_options(&BuildOptions::default())
        .expect("warm library build");
    assert_eq!(warm.counters().compiled, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_eq!(warm.counters().relinked, 0);

    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn source_changed_after_graph_creation_is_rejected_without_relabelling_sidecar() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let sidecar = fixture.base.with_extension("fpascu");
    let previous = fs::read(&sidecar).expect("initial sidecar");
    let project = load_project(&fixture.manifest).expect("project loading");
    let graph = build_unit_graph(&project.source_files, &project.link_meta).expect("unit graph");
    let selection = resolve_library_units(&graph).expect("library units");

    write(&fixture.base, &base_source("return Value + 100", 2));
    let error = build_library_units(&graph, &selection, &BuildOptions::default())
        .err()
        .expect("stale graph must fail");

    assert!(error.to_string().contains("changed after the build graph"));
    assert_eq!(
        fs::read(&sidecar).expect("preserved sidecar"),
        previous,
        "a stale graph must not publish an artifact under the changed source identity"
    );
    fs::remove_dir_all(&fixture.root).ok();
}
