#![allow(
    dead_code,
    reason = "each integration-test binary uses a different subset of transcript helpers"
)]

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::Value;
use tower_lsp_server::ls_types::Uri;

pub struct Transcript {
    pub output: Output,
    pub messages: Vec<Value>,
}

type MessageFactory = Box<dyn Fn(&[Value]) -> Value>;

pub enum TranscriptStep {
    Message(Value),
    MessageFrom(MessageFactory),
    Send(Value),
    Action(Box<dyn Fn()>),
    Wait(Duration),
}

pub fn run(messages: &[Value]) -> Transcript {
    let steps = messages
        .iter()
        .cloned()
        .map(TranscriptStep::Message)
        .collect::<Vec<_>>();
    run_script(&steps)
}

pub fn run_script(steps: &[TranscriptStep]) -> Transcript {
    let mut server = RunningServer::start();
    let mut messages = Vec::new();
    for step in steps {
        match step {
            TranscriptStep::Message(message) => {
                server.send(message);
                if let Some(id) = message.get("id") {
                    server.read_through_response(id, &mut messages);
                }
            }
            TranscriptStep::MessageFrom(message) => {
                let message = message(&messages);
                server.send(&message);
                if let Some(id) = message.get("id") {
                    server.read_through_response(id, &mut messages);
                }
            }
            TranscriptStep::Send(message) => server.send(message),
            TranscriptStep::Action(action) => action(),
            TranscriptStep::Wait(duration) => std::thread::sleep(*duration),
        }
    }
    let (output, remaining) = server.finish();
    messages.extend(parse_frames(&remaining));
    Transcript { output, messages }
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

    fn read_through_response(&mut self, expected_id: &Value, messages: &mut Vec<Value>) {
        loop {
            let message = self.read_message();
            let is_response =
                message.get("method").is_none() && message.get("id") == Some(expected_id);
            if message.get("method").is_some()
                && let Some(id) = message.get("id")
            {
                self.send(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": null
                }));
            }
            messages.push(message);
            if is_response {
                break;
            }
        }
    }

    fn finish(mut self) -> (Output, Vec<u8>) {
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
        (
            Output {
                status,
                stdout: Vec::new(),
                stderr,
            },
            remaining_stdout,
        )
    }
}

pub fn initialize(id: i64) -> Value {
    initialize_with_root(id, None)
}

pub fn initialize_with_root(id: i64, root_uri: Option<&str>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
            "capabilities": {
                "workspace": {
                    "workspaceEdit": {"documentChanges": true}
                }
            }
        }
    })
}

pub fn initialize_without_document_changes(id: i64, root_uri: Option<&str>) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": root_uri,
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
    let response = messages
        .iter()
        .find(|message| message.get("id") == Some(&Value::from(id)));
    assert!(response.is_some(), "missing response id {id}: {messages:?}");
    response.expect("response presence checked")
}

pub fn notifications<'a>(messages: &'a [Value], method: &str) -> Vec<&'a Value> {
    messages
        .iter()
        .filter(|message| message.get("method") == Some(&Value::from(method)))
        .collect()
}

pub fn parse_frames(stdout: &[u8]) -> Vec<Value> {
    let mut cursor = 0;
    let mut messages = Vec::new();
    while cursor < stdout.len() {
        let header_end = stdout[cursor..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|position| cursor + position);
        assert!(
            header_end.is_some(),
            "stdout contains unframed bytes: {:?}",
            &stdout[cursor..]
        );
        let header_end = header_end.expect("frame header presence checked");
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

pub struct TempDirectory {
    path: PathBuf,
}

impl TempDirectory {
    pub fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("fpas-lsp-{label}-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&path).expect("temporary fixture directory");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, path: impl AsRef<Path>, source: &str) -> PathBuf {
        let path = self.path.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture parent directory");
        }
        std::fs::write(&path, source).expect("fixture source");
        path
    }

    pub fn uri(&self, path: impl AsRef<Path>) -> String {
        Uri::from_file_path(self.path.join(path))
            .expect("file URI")
            .to_string()
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}
