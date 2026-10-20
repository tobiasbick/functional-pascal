//! Process options preserve typed results across source, cached units and portable images.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures assert process outcomes"
)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-process-options-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("main.fpas"), source).expect("source");
        fs::write(root.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
        Self(root)
    }
    fn invoke(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process")
    }
    fn run(&self, args: &[&str]) -> Output {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

#[test]
#[cfg(unix)]
fn options_and_partial_errors_survive_cached_units_and_source_free_images() {
    let source = r#"program ProcessImages;
uses ProcessBridge as Bridge, Std.Proc as Proc, Std.Test as Test;
begin
  const Defaults: Bridge.Options := Bridge.Defaults();
  Test.AssertTrue(Defaults.WorkingDirectory is None);
  Test.AssertEquals(0, Defaults.Environment.Length());
  Test.AssertEquals('', Defaults.Input);
  Test.AssertTrue(Defaults.TimeoutMillis is None);
  Test.AssertEquals(1048576, Defaults.MaxOutputBytes);
  Test.AssertTrue(Defaults.CancellationToken is None);
  const Options := Defaults with
    Input := 'input';
    Environment := [Proc.EnvironmentVariable(Name := 'FPAS_OPTIONS_TEST', Value := Some('configured'))];
  end with;
  const Output := Bridge.Capture('sh', ['-c', 'cat; printf "$FPAS_OPTIONS_TEST"'], Options).Unwrap();
  Test.AssertEquals('inputconfigured', Output.Stdout);
  Test.AssertEquals(0, Output.ExitCode);
  const Limited := Defaults with MaxOutputBytes := 4; end with;
  case Bridge.Capture('sh', ['-c', 'printf abcdefgh'], Limited) of
    when Ok(_): Test.Fail('Expected typed output limit');
    when Error(Proc.ProcError.LimitExceeded(const Operation, const Resource, const Limit, const Stdout, const Stderr)):
      Test.AssertEquals('Std.Proc.RunCaptureWithOptions', Operation);
      Test.AssertEquals('stdout bytes', Resource);
      Test.AssertEquals(4, Limit);
      Test.AssertEquals('abcd', Stdout);
      Test.AssertEquals('', Stderr);
    when Error(const Failure): Test.Fail(Failure.Message());
  end case;
end."#;
    let fixture = Fixture::new(source);
    fs::write(fixture.0.join("ProcessBridge.fpas"), r#"unit ProcessBridge;
uses Std.Proc as Proc;
public type Options = Proc.ProcessOptions;
public type Failure = Proc.ProcError;
public function Defaults(): Options;
begin return Proc.DefaultOptions(); end function;
public function Capture(Command: string; Args: array of string; Options: Proc.ProcessOptions): result of (Proc.ProcessOutput, Failure);
begin return Proc.RunCaptureWithOptions(Command, Args, Options); end function;
end unit;"#).expect("bridge");
    fixture.run(&["run", "app.fpasprj"]);
    let cache = fixture.0.join("ProcessBridge.fpascu");
    let bytes = fs::read(&cache).expect("compiled bridge");
    let modified = fs::metadata(&cache)
        .expect("metadata")
        .modified()
        .expect("mtime");
    fs::write(
        fixture.0.join("main.fpas"),
        format!("{source}\n// Caller changed.\n"),
    )
    .expect("caller change");
    fixture.run(&["run", "app.fpasprj"]);
    assert_eq!(bytes, fs::read(&cache).expect("cached bridge"));
    assert_eq!(
        modified,
        fs::metadata(&cache)
            .expect("metadata")
            .modified()
            .expect("mtime")
    );
    for file in [
        "main.fpas",
        "ProcessBridge.fpas",
        "ProcessBridge.fpascu",
        "ProcessBridge.fpascu.lock",
        "app.fpascp.lock",
    ] {
        let path = fixture.0.join(file);
        if path.exists() {
            fs::remove_file(path).expect("remove source and sidecars");
        }
    }
    fixture.run(&["run", "--std-lib", "absent-library", "app.fpascp"]);
}

#[test]
fn seeded_scheduling_allows_pure_defaults() {
    let fixture = Fixture::new(
        "program Defaults; uses Std.Proc as Proc, Std.Test as Test; begin const Options := Proc.DefaultOptions(); Test.AssertEquals(1048576, Options.MaxOutputBytes); end.",
    );
    let output = fixture.run(&["debug", "app.fpasprj", "--schedule-seed", "42"]);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["outcome"], "completed");
    assert_eq!(report["schedule"]["status"], "controlled");
}

#[test]
#[cfg(unix)]
fn seeded_scheduling_rejects_processes_before_spawn() {
    let fixture = Fixture::new(
        "program Effect; uses Std.Proc as Proc; begin discard Proc.RunWithOptions('sh', ['-c', 'touch spawned'], Proc.DefaultOptions()).Unwrap(); end.",
    );
    let output = fixture.invoke(&["debug", "app.fpasprj", "--schedule-seed", "42"]);
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["schedule"]["status"], "rejected");
    assert!(!fixture.0.join("spawned").exists());
}

#[test]
#[cfg(unix)]
fn options_and_legacy_runs_forward_child_stderr_as_json() {
    for expression in [
        "Proc.Run('sh', ['-c', 'echo child-error >&2'])",
        "Proc.RunWithOptions('sh', ['-c', 'echo child-error >&2'], Proc.DefaultOptions())",
    ] {
        let fixture = Fixture::new(&format!(
            "program Forward; uses Std.Proc as Proc, Std.Test as Test; begin Test.AssertEquals(0, {expression}.Unwrap()); end."
        ));
        let output = fixture.run(&["run", "--diagnostics", "json", "app.fpasprj"]);
        let text = String::from_utf8_lossy(&output.stderr);
        let records: Vec<serde_json::Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON stderr record"))
            .collect();
        assert_eq!(records.len(), 1, "{text}");
        assert_eq!(records[0]["kind"], "program-output");
        assert_eq!(records[0]["stream"], "stderr");
        assert_eq!(records[0]["text"], "child-error");
    }
}

#[test]
#[cfg(unix)]
fn killed_debug_worker_cleans_reparented_native_descendants() {
    let fixture = Fixture::new(
        "program WorkerDeath; uses Std.Proc as Proc; begin discard Proc.Run('sh', ['-c', '(printf ready > started; sleep 2; printf leaked > escaped) & while [ ! -f started ]; do sleep 0.01; done; kill -KILL $PPID; wait']).Unwrap(); end.",
    );
    let output = fixture.invoke(&["debug", "app.fpasprj"]);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("worker failure JSON");
    assert_eq!(report["outcome"], "error");
    assert_eq!(report["capture"]["incomplete"], true);
    assert_eq!(
        fs::read_to_string(fixture.0.join("started")).expect("descendant reached handshake"),
        "ready"
    );
    // A surviving descendant has enough time to create its marker even when the
    // debug worker's death caused the CLI to return before the child exits.
    std::thread::sleep(std::time::Duration::from_millis(2200));
    assert!(!fixture.0.join("escaped").exists());
}
