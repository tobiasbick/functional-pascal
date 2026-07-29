#![allow(
    dead_code,
    reason = "each integration-test binary uses a different subset of transcript helpers"
)]

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use serde_json::Value;

pub struct Transcript {
    pub output: Output,
    pub messages: Vec<Value>,
}

pub fn run(messages: &[Value]) -> Transcript {
    let mut server = RunningServer::start();
    let mut responses = Vec::new();
    for message in messages {
        server.send(message);
        if message.get("id").is_some() {
            responses.push(server.read_message());
        }
    }
    let output = server.finish();
    Transcript {
        output,
        messages: responses,
    }
}

pub fn frame(value: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(value).expect("serialize test message");
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend_from_slice(&body);
    frame
}

struct RunningServer {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl RunningServer {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_fpas-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start fpas-lsp");
        let stdin = child.stdin.take().expect("server stdin");
        let stdout = BufReader::new(child.stdout.take().expect("server stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, value: &Value) {
        self.stdin
            .write_all(&frame(value))
            .expect("write LSP frame");
        self.stdin.flush().expect("flush LSP frame");
    }

    fn read_message(&mut self) -> Value {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line).expect("read LSP header");
            assert!(!line.is_empty(), "server stdout ended before a response");
            if line == "\r\n" {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length:") {
                content_length = Some(value.trim().parse::<usize>().expect("content length"));
            }
        }
        let mut body = vec![0; content_length.expect("Content-Length response header")];
        self.stdout
            .read_exact(&mut body)
            .expect("read complete LSP response");
        serde_json::from_slice(&body).expect("valid JSON LSP response")
    }

    fn finish(mut self) -> Output {
        drop(self.stdin);
        let mut remaining_stdout = Vec::new();
        self.stdout
            .read_to_end(&mut remaining_stdout)
            .expect("read remaining stdout");
        let status = self.child.wait().expect("wait for fpas-lsp");
        let mut stderr = Vec::new();
        self.child
            .stderr
            .take()
            .expect("server stderr")
            .read_to_end(&mut stderr)
            .expect("read server stderr");
        Output {
            status,
            stdout: remaining_stdout,
            stderr,
        }
    }
}

pub fn initialize(id: i64) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": null,
            "capabilities": {}
        }
    })
}

pub fn initialized() -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    })
}

pub fn shutdown(id: i64) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "shutdown",
        "params": null
    })
}

pub fn exit() -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "exit",
        "params": null
    })
}

pub fn response(messages: &[Value], id: i64) -> &Value {
    messages
        .iter()
        .find(|message| message.get("id") == Some(&Value::from(id)))
        .unwrap_or_else(|| panic!("missing response id {id}: {messages:?}"))
}
