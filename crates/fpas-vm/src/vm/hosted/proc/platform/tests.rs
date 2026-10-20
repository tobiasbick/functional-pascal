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
    let mut bytes = Vec::new();
    output.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"probe");
    bytes.clear();
    errors.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"error");
    child.terminate_group().unwrap();
    assert_eq!(child.wait().unwrap(), status);
}
