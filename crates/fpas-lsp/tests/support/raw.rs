use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::support::Transcript;

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

fn parse_frames(stdout: &[u8]) -> Vec<Value> {
    let mut cursor = 0;
    let mut messages = Vec::new();
    while cursor < stdout.len() {
        let header_end = stdout[cursor..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|position| cursor + position)
            .unwrap_or_else(|| panic!("stdout contains unframed bytes: {:?}", &stdout[cursor..]));
        let header = std::str::from_utf8(&stdout[cursor..header_end]).expect("ASCII LSP header");
        let content_length = header
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length:")
                    .map(str::trim)
                    .map(str::parse::<usize>)
            })
            .expect("Content-Length header")
            .expect("numeric Content-Length");
        let body_start = header_end + 4;
        let body_end = body_start + content_length;
        assert!(
            body_end <= stdout.len(),
            "LSP body length exceeds stdout bytes"
        );
        messages.push(
            serde_json::from_slice(&stdout[body_start..body_end]).expect("valid JSON LSP body"),
        );
        cursor = body_end;
    }
    messages
}
