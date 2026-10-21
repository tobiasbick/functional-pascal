//! Collection overrides survive real run, isolated test and batch-debug execution.
#![allow(
    clippy::expect_used,
    reason = "process fixtures assert on filesystem and command results"
)]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-collection-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("fixture directory");
        fs::write(
            root.join("cycles_test.fpas"),
            r#"
program Cycles;
function Make(): function(): integer;
begin
  var Count := 0;
  var Current: function(): integer := function(): integer begin return 0; end function;
  Current := function(): integer begin
    Count := Count + 1;
    if Count < 0 then return Current(); end if;
    return Count;
  end function;
  return Current;
end function;
begin
  const Keep := Make();
  for I: integer := 1 to 30 do
    const Garbage := Make();
    if Garbage() <> 1 then panic('garbage'); end if;
    if Keep() <> I then panic('live'); end if;
  end for;
end.
"#,
        )
        .expect("cycle source");
        fs::write(root.join("app.fpasprj"), "[project]\nname='app'\nkind='program'\nmain='cycles_test.fpas'\n[sources]\ninclude=['cycles_test.fpas']\n").expect("program manifest");
        Self(root)
    }
    fn command(&self, mode: &str) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_fpas"));
        let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
        cmd.current_dir(&self.0)
            .arg(mode)
            .arg("--std-lib")
            .arg(library)
            .args([
                "--gc-cell-threshold=1",
                "--gc-mutation-threshold",
                "1",
                "--gc-poll-interval=1",
            ]);
        cmd
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
#[test]
fn source_cached_image_and_isolated_test_keep_cycles_and_live_values_correct() {
    let fixture = Fixture::new();
    success(
        fixture
            .command("run")
            .arg("cycles_test.fpas")
            .output()
            .expect("run source"),
    );
    success(
        fixture
            .command("run")
            .arg("cycles_test.fpas")
            .output()
            .expect("cached run"),
    );
    success(
        fixture
            .command("run")
            .arg("app.fpasprj")
            .output()
            .expect("project run"),
    );
    assert!(fixture.0.join("app.fpascp").is_file());
    success(
        fixture
            .command("run")
            .arg("app.fpascp")
            .output()
            .expect("compiled image run"),
    );
    let test = success(
        fixture
            .command("test")
            .args(["cycles_test.fpas", "--report", "json"])
            .output()
            .expect("isolated test"),
    );
    let report: serde_json::Value = serde_json::from_slice(&test.stdout).expect("test report");
    assert_eq!(report["summary"]["passed"], 1);
    assert_eq!(report["summary"]["failed"], 0);
}
#[test]
fn batch_debug_and_protocol_launch_accept_the_same_override_set() {
    let fixture = Fixture::new();
    let batch = success(
        fixture
            .command("debug")
            .arg("cycles_test.fpas")
            .output()
            .expect("batch"),
    );
    let report: serde_json::Value = serde_json::from_slice(&batch.stdout).expect("batch report");
    assert_eq!(report["outcome"], "completed");
    // EOF closes the transport without running a long-lived debuggee.
    for protocol in ["jsonl", "dap"] {
        success(
            fixture
                .command("debug")
                .arg("cycles_test.fpas")
                .args(["--protocol", protocol])
                .stdin(std::process::Stdio::null())
                .output()
                .expect("protocol launch"),
        );
    }
}
