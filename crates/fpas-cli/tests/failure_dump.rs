//! Real CLI boundaries for opt-in failure reports and atomic new-file publication.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "process fixtures assert filesystem and JSON behavior"
)]

#[path = "failure_dump/activation.rs"]
mod activation;
#[path = "failure_dump/limits.rs"]
mod limits;
#[path = "failure_dump/native.rs"]
mod native;
#[path = "failure_dump/publication.rs"]
mod publication;
#[path = "failure_dump/values.rs"]
mod values;

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
use std::{
    io::{BufRead, BufReader},
    process::{Child, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

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
        self.command().args(args).output().expect("CLI process")
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

struct Process(Option<Child>);
impl Process {
    fn spawn(fixture: &Fixture, args: &[&str]) -> Self {
        let mut command = fixture.command();
        command.args(args);
        Self::command(command)
    }
    fn command(mut command: Command) -> Self {
        Self(Some(
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("start controlled CLI"),
        ))
    }
    fn ready(&mut self) {
        let stdout = self
            .0
            .as_mut()
            .expect("CLI child")
            .stdout
            .take()
            .expect("piped stdout");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
            let _ = sender.send(result);
        });
        assert_eq!(
            receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("runtime barrier")
                .expect("stdout barrier"),
            "ready\n"
        );
    }
    fn finish(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if self
                .0
                .as_mut()
                .expect("CLI child")
                .try_wait()
                .expect("poll controlled CLI")
                .is_some()
            {
                break;
            }
            assert!(Instant::now() < deadline, "controlled CLI did not exit");
            std::thread::sleep(Duration::from_millis(2));
        }
        self.0
            .take()
            .expect("CLI child")
            .wait_with_output()
            .expect("controlled output")
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
