//! Integration tests for enforcing FPAS test-process timeouts.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "timeout integration fixtures require direct process and filesystem assertions"
)]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const CONTROL_SETUP_TIMEOUT: Duration = Duration::from_secs(20);

#[path = "../test_support/process.rs"]
mod process;

fn write(path: &Path, text: &str) {
    fs::write(path, text).expect("fixture must be written");
}

fn timed_test_command(root: &Path, test_file: &Path, timeout: &str) -> Command {
    let fpas = Path::new(env!("CARGO_BIN_EXE_fpas"));
    let standard_library = fpas.parent().expect("binary directory").join("lib");
    let scratch = root.join("scratch");
    fs::create_dir(&scratch).expect("scratch directory");

    let mut command = Command::new(fpas);
    command
        .current_dir(root)
        .args(["test", "--timeout", timeout, "--std-lib"])
        .arg(standard_library)
        .arg(test_file)
        .env("TMP", &scratch)
        .env("TEMP", &scratch)
        .env("TMPDIR", &scratch)
        .stdin(std::process::Stdio::null());
    command
}

fn run_timed_test(root: &Path, test_file: &Path) -> process::Process {
    process::Process::spawn(
        timed_test_command(root, test_file, "1"),
        Duration::from_secs(10),
    )
    .expect("independently supervised fpas test")
}

fn supervisor_child_name() -> String {
    let module = module_path!()
        .split_once("::")
        .expect("grouped case module")
        .1;
    format!("{module}::supervisor_child")
}

#[test]
fn timeout_force_stops_blocking_sleep_and_cleans_worker_files() {
    let root = process::TempDir::new("fpas-timeout-integration").expect("fixture root");
    let test_file = root.join("sleep_test.fpas");
    write(
        &test_file,
        "program SleepTest;\nuses Std.Time as Time;\nbegin Time.Sleep(60000); end.",
    );

    let started = Instant::now();
    let mut child = run_timed_test(&root, &test_file);
    child
        .observe_exit()
        .expect("FPAS timeout must finish before the outer watchdog");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "non-cooperative timeout took {:?}",
        started.elapsed()
    );
    assert_eq!(
        fs::read_dir(root.join("scratch"))
            .expect("scratch directory")
            .count(),
        0,
        "isolated worker files must be removed"
    );
    let output = child
        .wait()
        .expect("timeout worker output after SUT cleanup assertions");
    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("TIMEOUT  sleep_test.fpas"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn timeout_terminates_processes_started_by_the_test() {
    let root = process::TempDir::new("fpas-timeout-integration").expect("fixture root");
    let test_file = root.join("process_test.fpas");
    let started = root.join("descendant-started.txt");
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").expect("descendant liveness listener");
    listener
        .set_nonblocking(true)
        .expect("bounded descendant accept");
    let executable = std::env::current_exe()
        .expect("child executable")
        .to_string_lossy()
        .replace('\'', "''");
    let case = supervisor_child_name();
    write(
        &test_file,
        &format!(
            "program ProcessTest;\nuses Std.Os.Proc as Proc;\nbegin discard Proc.Run('{executable}', ['--ignored', '--exact', '{case}', '--nocapture']).Unwrap(); end."
        ),
    );

    // Starting a real test executable includes worker and native-child startup.
    // Keep this separate from the shorter blocking-sleep timeout regression.
    let mut command = timed_test_command(&root, &test_file, "5");
    command
        .env("FPAS_SUPERVISOR_ROLE", "descendant")
        .env("FPAS_SUPERVISOR_RECEIPT", &started)
        .env(
            "FPAS_SUPERVISOR_ADDRESS",
            listener.local_addr().expect("liveness address").to_string(),
        );
    let mut child = process::Process::spawn(command, Duration::from_secs(15))
        .expect("supervised child-start timeout test");
    child
        .observe_exit()
        .expect("FPAS timeout must finish before the outer watchdog");
    if !started.is_file() {
        let output = child.wait().expect("capture missing-start diagnostics");
        panic!("descendant startup was never observed; cleanup is unproven: {output:?}");
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream
                    .set_nonblocking(false)
                    .expect("blocking descendant observer");
                break stream;
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!(
                "started descendant did not connect: {error}\n{}",
                child.diagnostics()
            ),
        }
    };
    // The SUT must close this independently owned socket before any fallback
    // cleanup by the outer process supervisor can mask a leaked descendant.
    assert_tree_stopped(vec![stream]);
    let output = child
        .wait()
        .expect("timeout worker output after SUT descendant assertion");
    assert_eq!(
        output.status.code(),
        Some(3),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "owned process-tree control launched by the outer watchdog regression"]
fn supervisor_child() {
    use std::io::Write;
    let setup_deadline = Instant::now() + CONTROL_SETUP_TIMEOUT;
    let role = std::env::var("FPAS_SUPERVISOR_ROLE").expect("control role");
    if role == "parent" {
        // Deliberately exceed the work watchdog's three seconds before any
        // connection/readiness receipt, so startup must use its own budget.
        thread::sleep(Duration::from_secs(4));
    }
    let address = std::env::var("FPAS_SUPERVISOR_ADDRESS").expect("supervisor address");
    let _liveness = std::net::TcpStream::connect(address).expect("liveness connection");
    let descendant = if role != "descendant" {
        let mut command = Command::new(std::env::current_exe().expect("control executable"));
        command
            .args([
                "--ignored",
                "--exact",
                &std::env::var("FPAS_SUPERVISOR_CASE").expect("control name"),
                "--nocapture",
            ])
            .env("FPAS_SUPERVISOR_ROLE", "descendant");
        if role == "early" {
            command.env("FPAS_SUPERVISOR_QUIET", "1");
        }
        let mut command = process_wrap::std::CommandWrap::from(command);
        #[cfg(target_os = "linux")]
        if role != "early" {
            command.wrap(process_wrap::std::ProcessSession);
        }
        #[cfg(windows)]
        if role == "parent" {
            command.wrap(process_wrap::std::JobObject);
        }
        Some(
            command
                .spawn()
                .expect("descendant with independent session/job"),
        )
    } else {
        None
    };
    if role == "descendant" {
        fs::write(
            std::env::var("FPAS_SUPERVISOR_RECEIPT").expect("descendant receipt"),
            "ready",
        )
        .expect("descendant startup receipt");
    }
    if role != "early" && std::env::var_os("FPAS_SUPERVISOR_QUIET").is_none() {
        println!("SUPERVISOR_READY");
    }
    eprintln!("supervisor original stderr");
    std::io::stdout().flush().expect("flush control readiness");
    if matches!(role.as_str(), "leaky" | "early") {
        let variable = if role == "early" {
            "FPAS_SUPERVISOR_RECEIPT"
        } else {
            "FPAS_SUPERVISOR_RELEASE"
        };
        let release = std::env::var(variable).expect("control release path");
        while !Path::new(&release).exists() {
            // The root's release barrier belongs to setup too. The parent
            // independently enforces the overall twenty-second setup deadline.
            assert!(Instant::now() < setup_deadline, "control release deadline");
            thread::sleep(Duration::from_millis(2));
        }
        // Intentionally return without waiting: the test must observe the leak
        // before the independent supervisor performs its fallback teardown.
        drop(descendant);
        return;
    }
    thread::sleep(Duration::from_secs(60));
    if let Some(mut descendant) = descendant {
        let _ = descendant.wait();
    }
}

fn controlled_tree(
    role: &str,
    timeout: Duration,
) -> (process::TempDir, process::Process, Vec<std::net::TcpStream>) {
    use std::net::{TcpListener, TcpStream};

    let setup_deadline = Instant::now() + CONTROL_SETUP_TIMEOUT;
    let root = process::TempDir::new("fpas-watchdog-control").expect("control root");
    let listener = TcpListener::bind("127.0.0.1:0").expect("liveness listener");
    listener.set_nonblocking(true).expect("bounded accept");
    let case = supervisor_child_name();
    let mut command = Command::new(std::env::current_exe().expect("control executable"));
    command
        .args(["--ignored", "--exact", &case, "--nocapture"])
        .env("FPAS_SUPERVISOR_CASE", &case)
        .env(
            "FPAS_SUPERVISOR_ADDRESS",
            listener.local_addr().expect("liveness address").to_string(),
        )
        .env("FPAS_SUPERVISOR_ROLE", role)
        .env("FPAS_SUPERVISOR_RECEIPT", root.join("descendant-ready"))
        .env("FPAS_SUPERVISOR_RELEASE", root.join("release"));
    let remaining = setup_deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .expect("control setup deadline before spawn");
    let mut child = process::Process::spawn(command, remaining).expect("controlled tree");
    child.limit_remaining_time(setup_deadline.saturating_duration_since(Instant::now()));
    if role != "early" {
        child
            .wait_for_stdout_line("SUPERVISOR_READY")
            .expect("control readiness");
    }
    let mut streams: Vec<TcpStream> = Vec::new();
    while streams.len() < 2 {
        assert!(
            Instant::now() < setup_deadline,
            "control setup connection deadline"
        );
        match listener.accept() {
            Ok((stream, _)) => {
                stream
                    .set_nonblocking(false)
                    .expect("blocking liveness stream");
                streams.push(stream);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2));
            }
            Err(error) => panic!("parent and descendant must connect: {error}"),
        }
    }
    while !root.join("descendant-ready").is_file() {
        assert!(
            Instant::now() < setup_deadline,
            "control descendant receipt deadline"
        );
        thread::sleep(Duration::from_millis(2));
    }
    if role != "early" {
        // Both participants must have emitted readiness while the root's release
        // barrier still holds, so the supervisor owns the independent session.
        child
            .wait_for_stdout_line("SUPERVISOR_READY")
            .expect("second participant readiness");
    }
    assert!(
        Instant::now() < setup_deadline,
        "control setup exceeded its overall deadline"
    );
    // The work phase gets at most the requested time, and at most what is left
    // of the original overall setup budget. Neither call can extend a deadline.
    child.limit_remaining_time(setup_deadline.saturating_duration_since(Instant::now()));
    child.limit_remaining_time(timeout);
    (root, child, streams)
}

fn assert_tree_stopped(streams: Vec<std::net::TcpStream>) {
    use std::io::Read;
    for mut stream in streams {
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("bounded liveness read");
        match stream.read(&mut [0]) {
            Ok(0) => {}
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
            result => panic!("owned process is still alive: {result:?}"),
        }
    }
}

#[test]
fn outer_watchdog_stops_a_stuck_process_tree_and_preserves_diagnostics() {
    let (_root, child, streams) = controlled_tree("parent", Duration::from_secs(3));
    let started = Instant::now();
    let error = child
        .wait()
        .expect_err("stuck tree must reach independent deadline");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        error.to_string().contains("supervisor original stderr"),
        "{error}"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_tree_stopped(streams);
}

#[test]
fn normal_leader_exit_keeps_descendants_observable_until_fixture_teardown() {
    use std::io::Read;
    let (root, mut child, mut streams) = controlled_tree("leaky", Duration::from_secs(8));
    fs::write(root.join("release"), "exit leader").expect("release leaky leader");
    child
        .observe_exit()
        .expect("observe leader without cleaning its descendants");
    for stream in &mut streams {
        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("bounded leak observation");
    }
    // The root connection closes naturally; its descendant deliberately survives.
    let mut alive = 0;
    for stream in &mut streams {
        match stream.read(&mut [0]) {
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                alive += 1
            }
            Ok(0) => {}
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
            result => panic!("unexpected leak observation: {result:?}"),
        }
    }
    assert_eq!(
        alive, 1,
        "fallback cleanup must not hide the SUT leak before the assertion"
    );
    let output = child
        .wait()
        .expect("fallback cleanup after leak observation");
    assert!(output.status.success(), "{output:?}");
    assert_tree_stopped(streams);
}

#[test]
fn early_exit_before_readiness_cleans_descendant_held_pipes() {
    let (_root, mut child, streams) = controlled_tree("early", Duration::from_secs(3));
    let started = Instant::now();
    let error = child
        .wait_for_stdout_line("SUPERVISOR_READY")
        .expect_err("leader never emitted readiness");
    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    assert!(
        error.to_string().contains("supervisor original stderr"),
        "{error}"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    assert_tree_stopped(streams);
}
