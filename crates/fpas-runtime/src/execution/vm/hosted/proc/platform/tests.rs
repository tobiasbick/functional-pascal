use std::io::{Read, Write};
use std::process::{Command, Stdio};

use super::GroupChild;

#[test]
fn unix_pipe_probes_preserve_data_and_only_report_eof_after_exit() {
    let mut command = Command::new("sh");
    command.args([
        "-c",
        "IFS= read -r line; printf '%s' \"$line\"; printf error >&2",
    ]);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = GroupChild::spawn(command).unwrap();
    let mut input = child.take_stdin().unwrap().unwrap();
    let mut output = child.take_stdout().unwrap().unwrap();
    let mut errors = child.take_stderr().unwrap().unwrap();

    // The shell cannot print or exit before receiving the newline. This checks
    // readiness without a timer or an assumed scheduling order.
    assert_eq!(
        output.read(&mut [0]).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(child.try_wait().unwrap().is_none());
    input.write_all(b"probe\n").unwrap();
    input.flush().unwrap();
    drop(input);

    let status = child.wait().unwrap();
    assert!(status.success());
    assert_eq!(child.try_wait().unwrap(), Some(status));
    assert_eq!(drain(&mut output), b"probe");
    assert_eq!(drain(&mut errors), b"error");
    child.terminate_group().unwrap();
    assert_eq!(child.wait().unwrap(), status);
}

fn drain(reader: &mut impl Read) -> Vec<u8> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut result = Vec::new();
    let mut bytes = [0; 64];
    loop {
        match reader.read(&mut bytes) {
            Ok(0) => return result,
            Ok(count) => result.extend_from_slice(&bytes[..count]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                // Concurrent fork/exec can briefly retain another child's pipe
                // descriptors until CLOEXEC closes them. Leader exit alone does
                // not make the next nonblocking read an EOF observation.
                assert!(
                    std::time::Instant::now() < deadline,
                    "pipe never reached EOF"
                );
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(error) => panic!("pipe read: {error}"),
        }
    }
}
