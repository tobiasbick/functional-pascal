use std::io::Write;
use std::process::{Command, Stdio};

use crate::support::{Transcript, parse_frames};

pub fn run_frames(frames: &[Vec<u8>]) -> Transcript {
    let mut child = Command::new(env!("CARGO_BIN_EXE_fpas-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start fpas-lsp");

    {
        let stdin = child.stdin.as_mut().expect("server stdin");
        for frame in frames {
            stdin.write_all(frame).expect("write LSP frame");
        }
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait for fpas-lsp");
    let messages = parse_frames(&output.stdout);
    Transcript { output, messages }
}

pub fn frame_bytes(body: &[u8]) -> Vec<u8> {
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend_from_slice(body);
    frame
}
