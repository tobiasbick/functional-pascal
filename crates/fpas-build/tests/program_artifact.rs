#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "program artifact filesystem fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_build::{BuildOptions, ProgramArtifactTarget, build_program_artifact};
use fpas_project::{build_unit_graph_for_program, load_project};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("fpas-program-artifact-{}-{id}", std::process::id()))
}

fn write(path: &Path, source: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, source).expect("fixture source");
}

fn base_source(hidden_body: &str) -> String {
    format!(
        "unit Demo.Base;
         function Hidden(Value: integer): integer;
         begin {hidden_body} end;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;"
    )
}

struct Fixture {
    root: PathBuf,
    manifest: PathBuf,
    main: PathBuf,
    base: PathBuf,
    artifact: PathBuf,
}

impl Fixture {
    fn create() -> Self {
        let root = temp_dir();
        let manifest = root.join("demo.fpasprj");
        let main = root.join("src/main.fpas");
        let base = root.join("src/base.fpas");
        let artifact = root.join("demo.fpascp");
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
        write(&base, &base_source("return Value"));
        write(
            &root.join("src/consumer.fpas"),
            "unit Demo.Consumer;
             uses Demo.Base;
             public function Run(): integer;
             begin return AddOne(41) end;",
        );
        write(
            &main,
            "program Demo;
             uses Demo.Consumer, Std.Console;
             begin Std.Console.WriteLn(Run()) end.",
        );
        Self {
            root,
            manifest,
            main,
            base,
            artifact,
        }
    }

    fn build(&self) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
        let source = fs::read(&self.main).expect("main source");
        self.build_source_with_options(&source, &BuildOptions::default())
    }

    fn build_source(
        &self,
        source: &[u8],
    ) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
        self.build_source_with_options(source, &BuildOptions::default())
    }

    fn build_source_with_options(
        &self,
        source: &[u8],
        options: &BuildOptions,
    ) -> Result<fpas_build::BuiltProgram, fpas_build::BuildError> {
        let project = load_project(&self.manifest).expect("project loading");
        let graph =
            build_unit_graph_for_program(&self.main, &project.source_files, &project.link_meta)
                .expect("program unit graph");
        let source_paths = graph
            .source_paths()
            .iter()
            .map(|path| {
                path.strip_prefix(&self.root)
                    .expect("fixture-relative source")
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect::<Vec<_>>();
        build_program_artifact(
            &graph,
            ProgramArtifactTarget {
                path: &self.artifact,
                source,
                source_paths: &source_paths,
            },
            options,
        )
    }
}

fn assert_output(program: fpas_build::BuiltProgram, expected: &str) {
    let mut vm = fpas_vm::Vm::new(program.executable);
    vm.run().expect("compiled program execution");
    assert_eq!(vm.output().lines, [expected]);
}

#[test]
fn unchanged_program_artifact_is_reused_without_relinking() {
    let fixture = Fixture::create();

    let cold = fixture.build().expect("cold program artifact build");
    assert_eq!(cold.counters().compiled, 2);
    assert_eq!(cold.counters().relinked, 1);
    assert_eq!(cold.counters().program_image_reused, 0);
    assert_output(cold, "42");

    let warm = fixture.build().expect("warm program artifact build");
    assert_eq!(warm.counters().compiled, 0);
    assert_eq!(warm.counters().sidecar_reused, 2);
    assert_eq!(warm.counters().relinked, 0);
    assert_eq!(warm.counters().program_image_reused, 1);
    assert_output(warm, "42");

    let bytes = fs::read(&fixture.artifact).expect("compiled program");
    let image = fpas_program::decode(&bytes).expect("valid compiled program");
    for (path, hash) in image.source_paths().iter().zip(image.source_hashes()) {
        assert_eq!(
            *hash,
            fpas_program::Digest::of(
                fs::read(fixture.root.join(path)).expect("recorded source snapshot")
            ),
            "program image must bind `{path}` to its authoritative source bytes"
        );
    }
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn main_source_change_relinks_the_program() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let changed = b"program Demo;
                    uses Demo.Consumer, Std.Console;
                    begin Std.Console.WriteLn(Run() + 1) end.";
    write(
        &fixture.main,
        std::str::from_utf8(changed).expect("changed source"),
    );

    let rebuilt = fixture.build().expect("changed main build");

    assert_eq!(rebuilt.counters().compiled, 0);
    assert_eq!(rebuilt.counters().relinked, 1);
    assert_eq!(rebuilt.counters().program_image_reused, 0);
    assert_output(rebuilt, "43");
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn unit_implementation_change_relinks_without_rebuilding_consumers() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    write(&fixture.base, &base_source("return Value + 100"));

    let rebuilt = fixture.build().expect("unit implementation rebuild");

    assert_eq!(rebuilt.counters().compiled, 1);
    assert_eq!(rebuilt.counters().sidecar_reused, 1);
    assert_eq!(rebuilt.counters().relinked, 1);
    assert_eq!(rebuilt.counters().program_image_reused, 0);
    assert_output(rebuilt, "42");
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn compilation_option_change_relinks_the_program() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let source = fs::read(&fixture.main).expect("main source");
    let options = BuildOptions {
        options_hash: fpas_unit::Digest::of(b"changed program options"),
        ..BuildOptions::default()
    };

    let rebuilt = fixture
        .build_source_with_options(&source, &options)
        .expect("changed options build");

    assert_eq!(rebuilt.counters().compiled, 2);
    assert_eq!(rebuilt.counters().relinked, 1);
    assert_eq!(rebuilt.counters().program_image_reused, 0);
    assert_output(rebuilt, "42");
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn corrupt_program_artifact_is_rebuilt_and_replaced() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    fs::write(&fixture.artifact, b"corrupt program image").expect("corrupt artifact");

    let rebuilt = fixture.build().expect("corrupt artifact recovery");

    assert_eq!(rebuilt.counters().compiled, 0);
    assert_eq!(rebuilt.counters().relinked, 1);
    let bytes = fs::read(&fixture.artifact).expect("replaced artifact");
    fpas_program::decode(&bytes).expect("valid replacement");
    assert_output(rebuilt, "42");
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn failed_program_rebuild_preserves_the_previous_artifact() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let previous = fs::read(&fixture.artifact).expect("initial artifact");
    let invalid = b"program Demo;
                    uses Demo.Consumer, Std.Console;
                    begin Std.Console.WriteLn(Missing()) end.";

    assert!(fixture.build_source(invalid).is_err());
    assert_eq!(
        fs::read(&fixture.artifact).expect("preserved artifact"),
        previous
    );
    fs::remove_dir_all(&fixture.root).ok();
}

#[test]
fn non_program_source_is_rejected_before_cached_artifact_lookup() {
    let fixture = Fixture::create();
    fixture.build().expect("initial build");
    let previous = fs::read(&fixture.artifact).expect("initial artifact");
    let unit_source = b"unit Demo; public const Value: integer := 1;";

    let error = fixture
        .build_source(unit_source)
        .err()
        .expect("unit source must not build as a program artifact");

    assert!(error.to_string().contains("instead of a program"));
    assert_eq!(
        fs::read(&fixture.artifact).expect("preserved artifact"),
        previous
    );
    fs::remove_dir_all(&fixture.root).ok();
}
