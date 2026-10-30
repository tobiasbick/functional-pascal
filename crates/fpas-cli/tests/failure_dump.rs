//! Real CLI boundaries for opt-in failure reports and atomic new-file publication.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures assert filesystem and JSON behavior"
)]

#[path = "failure_dump/activation.rs"]
mod activation;
#[path = "failure_dump/end_to_end.rs"]
mod end_to_end;
#[path = "failure_dump/limits.rs"]
mod limits;
#[path = "failure_dump/native.rs"]
mod native;
#[path = "failure_dump/publication.rs"]
mod publication;
#[path = "failure_dump/values.rs"]
mod values;

use serde_json::Value;
use std::time::{Duration, Instant};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../test_support/process.rs"]
mod process;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(".temp-data")
            .join(format!(
                "cli-failure-dump-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&path).expect("fixture directory");
        Self(path.canonicalize().expect("fixture root"))
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture parents");
        fs::write(path, text).expect("fixture source");
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command.current_dir(&self.0);
        command
    }
    fn run(&self, args: &[&str]) -> Output {
        let mut command = self.command();
        command.args(args);
        process::capture(command, Duration::from_secs(15)).expect("bounded CLI process")
    }
    fn report(&self, path: &str) -> Value {
        let bytes = fs::read(self.0.join(path)).expect("published report");
        assert!(bytes.ends_with(b"\n"));
        assert!(bytes.len() <= 8 * 1024 * 1024);
        serde_json::from_slice(&bytes).expect("one complete JSON document")
    }
    fn project(&self, source: &str) {
        self.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/**/*.fpas']\n");
        self.write("src/main.fpas", source);
    }
    fn no_staging(&self) {
        fn check(path: &Path) {
            for entry in fs::read_dir(path).expect("fixture entries") {
                let entry = entry.expect("fixture entry");
                assert!(
                    !entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".fpas-failure-"),
                    "leaked staging file: {entry:?}"
                );
                if entry.file_type().expect("fixture type").is_dir() {
                    check(&entry.path());
                }
            }
        }
        check(&self.0);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn exit(output: &Output, expected: i32) {
    assert_eq!(
        output.status.code(),
        Some(expected),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn records(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON diagnostic line"))
        .collect()
}

struct Process {
    child: process::Process,
    consumed_ready: bool,
}
impl Process {
    fn spawn(fixture: &Fixture, args: &[&str]) -> Self {
        let mut command = fixture.command();
        command.args(args);
        Self::command(command)
    }
    fn command(command: Command) -> Self {
        Self {
            child: process::Process::spawn(command, Duration::from_secs(15))
                .expect("start controlled CLI"),
            consumed_ready: false,
        }
    }
    fn ready(&mut self) {
        assert_eq!(
            self.child
                .wait_for_stdout_line("ready")
                .expect("runtime barrier"),
            "ready\n"
        );
        self.consumed_ready = true;
    }
    fn finish(self) -> Output {
        let mut output = self.child.wait().expect("controlled output");
        if self.consumed_ready
            && let Some(stdout) = output.stdout.strip_prefix(b"ready\n")
        {
            output.stdout = stdout.to_vec();
        }
        output
    }
}
