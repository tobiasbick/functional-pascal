//! Process preflight and owned stdio supervision without shell timing assumptions.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "isolated child-process fixtures"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::failure::Kind;
use super::options::{DEFAULT_OUTPUT, MAX_IO, Options, Request};
use super::registry::Processes;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        loop {
            let path = std::env::temp_dir().join(format!(
                "fpas-process-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("fixture directory: {error}"),
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn shell(script: &str) -> Request<'_> {
    Request {
        command: "/bin/sh",
        arguments: vec!["-c", script],
        options: Options {
            timeout: Some(Duration::from_secs(10)),
            ..Options::default()
        },
        capture: true,
        inherit_input: false,
    }
}

#[test]
fn configuration_preflight_covers_budgets_duplicates_and_embedded_nul() {
    let mut request = Request {
        command: "unused",
        arguments: vec![],
        options: Options::default(),
        capture: true,
        inherit_input: false,
    };
    request.arguments = vec!["x"; 4097];
    assert!(matches!(
        request.validate().expect_err("arguments").kind,
        Kind::Limit("arguments", 4096)
    ));
    request.arguments.clear();
    request.options.environment = vec![("a", None); 1025];
    assert!(matches!(
        request.validate().expect_err("environment").kind,
        Kind::Limit("environment", 1024)
    ));
    request.options.environment = vec![("a", None), ("a", Some("b"))];
    assert!(matches!(
        request.validate().expect_err("duplicate").kind,
        Kind::Argument("Environment", _)
    ));
    for name in ["", "a=b", "a\0b"] {
        request.options.environment = vec![(name, None)];
        assert!(matches!(
            request.validate().expect_err("invalid name").kind,
            Kind::Argument("Environment", _)
        ));
    }
    request.options.environment.clear();
    request.command = "bad\0command";
    assert!(matches!(
        request.validate().expect_err("NUL command").kind,
        Kind::Argument("Command", _)
    ));
    request.command = "unused";
    let large = "a".repeat(1024 * 1024);
    request.arguments = vec![&large];
    assert!(matches!(
        request.validate().expect_err("configuration").kind,
        Kind::Limit("configuration bytes", 1048576)
    ));
    request.arguments.clear();
    let input = "a".repeat(MAX_IO + 1);
    request.options.input = &input;
    assert!(matches!(
        request.validate().expect_err("input").kind,
        Kind::Limit("input bytes", MAX_IO)
    ));
    request.options.input = "allowed\0input";
    request.validate().expect("stdin permits NUL");
    request.options.maximum = MAX_IO + 1;
    assert!(matches!(
        request.validate().expect_err("output cap").kind,
        Kind::Argument("MaxOutputBytes", _)
    ));
}

#[test]
fn environment_duplicates_follow_native_keys_without_full_case_expansion() {
    let mut request = Request {
        command: "unused",
        arguments: vec![],
        options: Options::default(),
        capture: true,
        inherit_input: false,
    };
    request.options.environment = vec![("ß", None), ("SS", None)];
    request
        .validate()
        .expect("distinct native environment names");
    request.options.environment = vec![("FPAS_CASE", None), ("fpas_case", None)];
    assert_eq!(request.validate().is_err(), cfg!(windows));
}

#[cfg(unix)]
#[test]
fn simultaneous_large_input_stdout_and_stderr_do_not_deadlock() {
    let input = "i".repeat(262_144);
    let mut request = shell("head -c 262144 /dev/zero | tr '\\000' e >&2 & cat; wait");
    request.options.input = &input;
    let output = Processes::new()
        .run(&request, None, || false)
        .expect("duplex child");
    assert_eq!(output.code, 0);
    assert_eq!(output.stdout, input);
    assert_eq!(output.stderr, "e".repeat(262_144));
}

#[cfg(unix)]
#[test]
fn a_child_that_closes_stdin_early_can_still_complete_successfully() {
    let input = "unread".repeat(65_536);
    let mut request = shell("exec 0<&-; printf done; exit 7");
    request.options.input = &input;
    let output = Processes::new()
        .run(&request, None, || false)
        .expect("early stdin closure is normal child behavior");
    assert_eq!(output.code, 7);
    assert_eq!(output.stdout, "done");
    assert!(output.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn exact_raw_output_caps_zero_and_lossy_error_prefixes_are_distinct() {
    let processes = Processes::new();
    let mut request = shell("printf ABC; printf xyz >&2");
    request.options.maximum = 3;
    let output = processes
        .run(&request, None, || false)
        .expect("exact stream caps");
    assert_eq!(output.stdout, "ABC");
    assert_eq!(output.stderr, "xyz");
    request.arguments[1] = "printf ABCD";
    let failure = processes
        .run(&request, None, || false)
        .err()
        .expect("over limit");
    assert!(matches!(failure.kind, Kind::Limit("stdout bytes", 3)));
    assert_eq!(failure.stdout, "ABC");
    request.arguments[1] = "printf '\\377\\376\\375\\374'";
    let failure = processes
        .run(&request, None, || false)
        .err()
        .expect("raw byte cap");
    assert!(matches!(failure.kind, Kind::Limit("stdout bytes", 3)));
    assert_eq!(failure.stdout, "\u{fffd}".repeat(3));
    request.options.maximum = 0;
    request.arguments[1] = ":";
    processes
        .run(&request, None, || false)
        .expect("empty output at zero cap");
    request.arguments[1] = "printf x";
    let failure = processes
        .run(&request, None, || false)
        .err()
        .expect("nonempty output at zero");
    assert!(matches!(failure.kind, Kind::Limit("stdout bytes", 0)));
    assert!(failure.stdout.is_empty());
}

#[cfg(unix)]
#[test]
fn cwd_environment_overlay_and_input_eof_follow_options() {
    let fixture = Fixture::new();
    let directory = fs::canonicalize(&fixture.0)
        .expect("canonical fixture")
        .to_string_lossy()
        .into_owned();
    let mut request = shell("pwd; printf '%s|%s' \"$FPAS_PROCESS_TEST\" \"${HOME-unset}\"; cat");
    request.options.directory = Some(&directory);
    request.options.environment = vec![("FPAS_PROCESS_TEST", Some("present")), ("HOME", None)];
    let output = Processes::new()
        .run(&request, None, || false)
        .expect("configured child");
    assert_eq!(output.stdout, format!("{directory}\npresent|unset"));
}

#[cfg(unix)]
#[test]
fn preflight_and_zero_timeout_or_precancellation_never_launch_a_child() {
    let fixture = Fixture::new();
    let marker = fixture.0.join("launched").to_string_lossy().into_owned();
    let mut request = shell(": > \"$1\"");
    request.arguments.extend(["fixture", &marker]);
    request.options.timeout = Some(Duration::ZERO);
    assert!(matches!(
        Processes::new()
            .run(&request, None, || false)
            .err()
            .expect("zero timeout")
            .kind,
        Kind::Timeout
    ));
    assert!(!std::path::Path::new(&marker).exists());
    request.options.timeout = None;
    assert!(matches!(
        Processes::new()
            .run(&request, None, || true)
            .err()
            .expect("pre-cancel")
            .kind,
        Kind::Cancelled
    ));
    assert!(!std::path::Path::new(&marker).exists());
    let processes = Processes::with_limit(1);
    let reservation = processes.reserve().expect("occupied slot");
    assert!(matches!(
        processes
            .run(&request, None, || false)
            .err()
            .expect("admission")
            .kind,
        Kind::Limit("processes", 1)
    ));
    assert!(!std::path::Path::new(&marker).exists());
    drop(reservation);
    request.command = "";
    assert!(matches!(
        processes
            .run(&request, None, || false)
            .err()
            .expect("empty command")
            .kind,
        Kind::Start(_)
    ));
}

#[cfg(unix)]
#[test]
fn cancellation_preserves_observed_output_and_cleans_the_owned_group() {
    let fixture = Fixture::new();
    let marker = fixture.0.join("started");
    let marker_text = marker.to_string_lossy().into_owned();
    let mut request = shell("printf prefix; : > \"$1\"; sleep 30");
    request.arguments.extend(["fixture", &marker_text]);
    let processes = Processes::new();
    let failure = processes
        .run(&request, None, || marker.exists())
        .err()
        .expect("cancel running child");
    assert!(matches!(failure.kind, Kind::Cancelled));
    assert_eq!(failure.stdout, "prefix");
    processes.wait_idle();
    let mut timeout = shell("sleep 30");
    timeout.options.timeout = Some(Duration::from_millis(50));
    assert!(matches!(
        processes
            .run(&timeout, None, || false)
            .err()
            .expect("active timeout")
            .kind,
        Kind::Timeout
    ));
    processes.wait_idle();
}

#[cfg(unix)]
#[test]
fn leader_exit_cleans_descendants_that_hold_capture_pipes() {
    let request = shell("sleep 30 & printf prefix");
    let output = Processes::new()
        .run(&request, None, || false)
        .expect("leader completion closes group pipes");
    assert_eq!(output.code, 0);
    assert_eq!(output.stdout, "prefix");
}

#[cfg(unix)]
#[test]
fn shutdown_between_reservation_and_attach_kills_the_new_child() {
    let processes = Processes::new();
    let reservation = processes.reserve().expect("reservation");
    processes.shutdown();
    let mut command = std::process::Command::new("/bin/sh");
    command.args(["-c", "sleep 30"]);
    reservation
        .process
        .spawn(command)
        .expect("attach observes shutdown");
    let status = reservation
        .process
        .reap()
        .expect("reap")
        .expect("child existed");
    assert!(!status.success());
    drop(reservation);
    processes.wait_idle();
}

#[cfg(unix)]
#[test]
fn legacy_stderr_receiver_can_reenter_vm_shutdown_without_self_deadlock() {
    let (program, diagnostics) = fpas_parser::parse("program Empty; begin end.");
    assert!(diagnostics.is_empty());
    let vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("empty VM"));
    let shutdown = vm.shutdown_handle();
    let receiver: fpas_std::ProgramStderr = Arc::new(move |_| shutdown.shutdown());
    let mut request = shell("printf 'ready\\n' >&2; sleep 30");
    request.capture = false;
    request.inherit_input = true;
    let failure = vm
        .hosted
        .processes
        .run(&request, Some(receiver), || false)
        .err()
        .expect("receiver requested shutdown");
    assert!(matches!(failure.kind, Kind::Cancelled));
    vm.hosted.wait_process_handles();
}

#[cfg(unix)]
#[test]
fn controlled_stderr_delivery_happens_after_group_cleanup_and_keeps_partial_errors() {
    let processes = Arc::new(Processes::new());
    let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let owner = Arc::clone(&processes);
    let receiver: fpas_std::ProgramStderr = Arc::new(move |line| {
        owner.shutdown();
        sink.lock().expect("sink").push(line.to_owned());
    });
    let mut request = shell("printf 'captured\\n' >&2");
    request.capture = false;
    let output = processes
        .run(&request, Some(receiver), || false)
        .expect("callback after successful cleanup");
    assert_eq!(output.stderr, "captured\n");
    assert_eq!(*observed.lock().expect("observed"), ["captured"]);
    processes.wait_idle();
    assert_eq!(DEFAULT_OUTPUT, 1_048_576);
}
