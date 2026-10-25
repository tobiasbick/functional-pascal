//! A real controlling terminal distinguishes foreground transfer from ordinary pipe inheritance.

use super::*;

#[test]
fn real_terminal_is_foregrounded_and_restored_after_login() {
    let fixture = Fixture::new();
    let pty = nix::pty::openpty(None, None).expect("pseudoterminal");
    let mut master = File::from(pty.master);
    let slave = File::from(pty.slave);
    use nix::fcntl::{FcntlArg, OFlag, fcntl};
    let flags = fcntl(&master, FcntlArg::F_GETFL).unwrap();
    fcntl(
        &master,
        FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK),
    )
    .unwrap();
    let mut command = Command::new("setsid");
    command
        .args(["--ctty", "--wait"])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "foreground::inherited_terminal_host",
            "--nocapture",
        ])
        .env("FPAS_TERMINAL_ROOT", &fixture.0)
        .stdin(slave.try_clone().unwrap())
        .stdout(slave.try_clone().unwrap())
        .stderr(slave);
    let mut child = command.spawn().expect("terminal host");
    drop(command);
    master.write_all(b"interactive input\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut output = Vec::new();
    let status = loop {
        let mut bytes = [0; 4096];
        match master.read(&mut bytes) {
            Ok(count) => output.extend_from_slice(&bytes[..count]),
            Err(error)
                if error.kind() == io::ErrorKind::WouldBlock
                    || error.raw_os_error() == Some(nix::libc::EIO) => {}
            Err(error) => panic!("terminal read: {error}"),
        }
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline || output.len() > 65536 {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "terminal host did not finish: {}",
                String::from_utf8_lossy(&output)
            );
        }
        std::thread::sleep(Duration::from_millis(1));
    };
    match master.read_to_end(&mut output) {
        Ok(_) => {}
        Err(error) if error.raw_os_error() == Some(nix::libc::EIO) => {}
        Err(error) => panic!("terminal drain: {error}"),
    }
    let text = String::from_utf8_lossy(&output);
    assert!(status.success(), "{text}");
    assert!(text.contains("FOREGROUND_RESTORED"), "{text}");
}

#[test]
#[ignore = "host fixture needs an isolated controlling terminal"]
fn inherited_terminal_host() {
    let root = std::env::var("FPAS_TERMINAL_ROOT").unwrap();
    let fixture = Fixture(PathBuf::from(root));
    let mut vm = fixture.vm(
        &format!(
            "const Status := {}.Unwrap(); Test.AssertEquals(7, Status.ExitCode.Unwrap());",
            RUN.replace("Some(15000)", "Some(5000)")
        ),
        "",
    );
    vm.set_interactive_process_terminal(
        InteractiveProcessTerminal::inherited().expect("actual terminal"),
    );
    vm.run().expect("foreground child can read terminal input");
    assert_eq!(
        nix::unistd::tcgetpgrp(io::stdin()).unwrap(),
        nix::unistd::getpgrp()
    );
    println!("FOREGROUND_RESTORED");
    // The parent owns the fixture directory; this process owns only its terminal session.
    std::mem::forget(fixture);
}
