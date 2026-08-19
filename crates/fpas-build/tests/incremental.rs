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
    let mut vm = fpas_vm::Vm::new(program.executable);
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
fn register_build_is_deterministic_and_recovers_every_sidecar_class() {
    let fixture = Fixture::create();
    let options = BuildOptions::default();

    let cold = fixture
        .build_with_options(&options)
        .expect("cold register build");
    assert_eq!(cold.counters().compiled, 2);
    let cold_executable = cold.executable.executable().clone();
    let base_sidecar = fixture.base.with_extension("fpascu");
    let consumer_sidecar = fixture.root.join("src/consumer.fpascu");
    let cold_object = decode(&fs::read(&base_sidecar).expect("cold base sidecar"))
        .expect("cold base envelope")
        .object;
    assert_output(cold, "42");

    let warm = fixture
        .build_with_options(&options)
        .expect("warm register build");
    assert_eq!(warm.counters().compiled, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_eq!(warm.executable.executable(), &cold_executable);
    assert_eq!(
        decode(&fs::read(&base_sidecar).expect("warm base sidecar"))
            .expect("warm base envelope")
            .object,
        cold_object,
        "identical register builds must retain byte-identical object payloads"
    );
    assert_output(warm, "42");

    fs::remove_file(&base_sidecar).expect("remove base sidecar");
    let missing = fixture
        .build_with_options(&options)
        .expect("missing register sidecar rebuild");
    assert_eq!(missing.counters().compiled, 1);
    assert_eq!(missing.counters().sidecar_reused, 1);

    let mut old = fs::read(&consumer_sidecar).expect("consumer sidecar");
    old[8..10].copy_from_slice(&(fpas_unit::FORMAT_VERSION - 1).to_le_bytes());
    fs::write(&consumer_sidecar, old).expect("old register sidecar fixture");
    let old = fixture
        .build_with_options(&options)
        .expect("old register sidecar rebuild");
    assert_eq!(old.counters().compiled, 1);
    assert_eq!(old.counters().sidecar_reused, 1);

    let mut corrupt =
        decode(&fs::read(&base_sidecar).expect("base sidecar")).expect("base envelope");
    corrupt.object = b"invalid register object".to_vec();
    corrupt.identity.object_hash = Digest::of(&corrupt.object);
    fs::write(&base_sidecar, encode(&corrupt).expect("corrupt envelope"))
        .expect("corrupt register sidecar fixture");
    let corrupt = fixture
        .build_with_options(&options)
        .expect("corrupt register sidecar rebuild");
    assert_eq!(corrupt.counters().compiled, 1);
    assert_eq!(corrupt.counters().sidecar_reused, 1);

    let mut incompatible_options = options;
    incompatible_options.bytecode_version = incompatible_options.bytecode_version.saturating_add(1);
    let incompatible = fixture
        .build_with_options(&incompatible_options)
        .expect("incompatible register sidecar rebuild");
    assert_eq!(incompatible.counters().compiled, 2);
    assert_eq!(incompatible.counters().sidecar_reused, 0);
    assert_output(incompatible, "42");

    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn register_build_runs_a_workspace_library_dependency() {
    let root = temp_dir();
    write(
        &root.join("suite.fpasworkspace"),
        "[workspace]\nname = \"suite\"\nmembers = [\"app/app.fpasprj\", \"lib/lib.fpasprj\"]\n",
    );
    let app_manifest = root.join("app/app.fpasprj");
    write(
        &app_manifest,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[dependencies]
workspace = ["core"]

[sources]
include = ["main.fpas"]
"#,
    );
    write(
        &root.join("app/main.fpas"),
        "program Demo;
         uses Demo.Consumer, Std.Console;
         begin Std.Console.WriteLn(Run()) end.",
    );
    write(
        &root.join("lib/lib.fpasprj"),
        r#"[project]
name = "core"
kind = "library"

[sources]
include = ["*.fpas"]

[exports]
units = ["Demo.Base", "Demo.Consumer"]
"#,
    );
    write(
        &root.join("lib/base.fpas"),
        "unit Demo.Base;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    write(
        &root.join("lib/consumer.fpas"),
        "unit Demo.Consumer;
         uses Demo.Base;
         public function Run(): integer;
         begin return AddOne(41) end;",
    );

    let project = load_project(&app_manifest).expect("workspace program project");
    let graph =
        build_unit_graph(&project.source_files, &project.link_meta).expect("workspace graph");
    let main = project.main.expect("workspace program main");
    let source = fs::read_to_string(main).expect("workspace program source");
    let (program, diagnostics) = fpas_parser::parse(&source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let selection = resolve_program_units(&graph, &program.uses).expect("workspace selection");
    let built = build_program(&graph, &selection, &program, &BuildOptions::default())
        .expect("workspace register build");

    assert_eq!(built.counters().compiled, 2);
    assert_output(built, "42");
    fs::remove_dir_all(root).ok();
}

#[test]
fn imported_qualified_enum_member_uses_persisted_backing_value() {
    let root = temp_dir();
    let manifest = root.join("enum.fpasprj");
    write(
        &manifest,
        r#"[project]
name = "enum-value"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write(
        &root.join("src/values.fpas"),
        "unit Demo.Values;
         public type State = enum Idle = 7; Ready; Done = 20; end;",
    );
    write(
        &root.join("src/main.fpas"),
        "program Demo;
         uses Demo.Values, Std.Console;
         begin Std.Console.WriteLn(Demo.Values.State.Ready) end.",
    );

    let project = load_project(&manifest).expect("enum project loading");
    let graph = build_unit_graph(&project.source_files, &project.link_meta).expect("enum graph");
    let main = project.main.expect("enum program main");
    let source = fs::read_to_string(main).expect("enum main source");
    let (program, diagnostics) = fpas_parser::parse(&source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let selection = resolve_program_units(&graph, &program.uses).expect("enum selection");
    let built = build_program(&graph, &selection, &program, &BuildOptions::default())
        .expect("enum register build");

    assert_output(built, "8");
    fs::remove_dir_all(root).ok();
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
fn old_sidecar_envelope_rebuilds_automatically_and_is_replaced() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let sidecar = fixture.base.with_extension("fpascu");
    let mut old = fs::read(&sidecar).expect("current sidecar");
    old[8..10].copy_from_slice(&(fpas_unit::FORMAT_VERSION - 1).to_le_bytes());
    fs::write(&sidecar, old).expect("old sidecar fixture");

    let rebuilt = fixture.build().expect("old format rebuild");
    assert_eq!(rebuilt.counters().compiled, 1);
    assert_eq!(rebuilt.counters().sidecar_reused, 1);
    assert_output(rebuilt, "42");
    let replacement = fs::read(&sidecar).expect("replacement sidecar");
    assert_eq!(
        u16::from_le_bytes([replacement[8], replacement[9]]),
        fpas_unit::FORMAT_VERSION
    );

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
