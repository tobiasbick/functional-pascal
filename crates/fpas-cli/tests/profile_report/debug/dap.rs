//! Native DAP transport proves profile ownership across stops and resume threads.

use super::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStdin, Stdio},
    sync::mpsc::{self, Receiver},
    time::{Duration, Instant},
};

struct Client {
    child: Child,
    input: Option<ChildStdin>,
    messages: Receiver<Value>,
    reader: Option<std::thread::JoinHandle<()>>,
    sequence: u64,
}

impl Client {
    fn new(fixture: &Fixture) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&fixture.0)
            .args([
                "debug",
                "main.fpas",
                "--protocol",
                "dap",
                "--profile",
                "debug.json",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("DAP process");
        let input = child.stdin.take();
        let output = child.stdout.take().expect("DAP stdout");
        let (sender, messages) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut output = BufReader::new(output);
            loop {
                let mut header = String::new();
                if output.read_line(&mut header).unwrap_or(0) == 0 {
                    break;
                }
                let length: usize = header
                    .trim()
                    .strip_prefix("Content-Length: ")
                    .expect("DAP framing remains intact")
                    .parse()
                    .expect("content length");
                assert!(length <= 16 * 1024 * 1024);
                let mut separator = String::new();
                output.read_line(&mut separator).expect("header separator");
                assert_eq!(separator, "\r\n");
                let mut body = vec![0; length];
                output.read_exact(&mut body).expect("DAP body");
                if sender
                    .send(serde_json::from_slice(&body).expect("DAP JSON"))
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            child,
            input,
            messages,
            reader: Some(reader),
            sequence: 0,
        }
    }

    fn send(&mut self, command: &str, arguments: Value) -> u64 {
        self.sequence += 1;
        let body =
            json!({"seq":self.sequence,"type":"request","command":command,"arguments":arguments})
                .to_string();
        let input = self.input.as_mut().expect("open DAP stdin");
        write!(input, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("DAP request");
        input.flush().expect("request flush");
        self.sequence
    }

    fn until(&self, predicate: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let message = self
                .messages
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("bounded DAP response");
            if predicate(&message) {
                return message;
            }
        }
    }

    fn finish(&mut self) {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.child.try_wait().expect("DAP status") {
                assert!(status.success(), "{status}");
                break;
            }
            assert!(Instant::now() < deadline, "DAP transport did not close");
            std::thread::sleep(Duration::from_millis(1));
        }
        self.reader
            .take()
            .expect("reader thread")
            .join()
            .expect("framed reader");
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.input.take();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

#[test]
fn dap_target_profiles_resume_and_inspection_but_keeps_client_pause_out_of_resume_time() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", SOURCE);
    let mut client = Client::new(&fixture);
    client.send("initialize", json!({}));
    client.send("launch", json!({"stopOnEntry":false}));
    client.send(
        "setFunctionBreakpoints",
        json!({"breakpoints":[{"name":"Marker"}]}),
    );
    client.send("configurationDone", json!({}));
    let stopped = client.until(|message| message["event"] == "stopped");
    assert_eq!(stopped["body"]["reason"], "breakpoint");
    assert!(
        !fixture.0.join("debug.json").exists(),
        "report is published after session cleanup"
    );
    std::thread::sleep(Duration::from_millis(250));
    let request = client.send(
        "stackTrace",
        json!({"threadId":stopped["body"]["threadId"]}),
    );
    let stack = client.until(|message| message["request_seq"] == request);
    assert_eq!(stack["success"], true);
    assert!(
        !stack["body"]["stackFrames"]
            .as_array()
            .expect("stack frames")
            .is_empty()
    );
    fixture.write(
        "main.fpas",
        &SOURCE.replace("return Total;", "return Total + 1;"),
    );
    let request = client.send("fpas/reload", json!({}));
    let reload = client.until(|message| message["request_seq"] == request);
    assert_eq!(reload["success"], false);
    assert!(reload.to_string().contains("profiling"), "{reload}");
    client.send("continue", json!({"threadId":stopped["body"]["threadId"]}));
    client.until(|message| message["event"] == "terminated");
    client.finish();
    let report = fixture.report("debug.json");
    assert_target(&report, "dap");
    assert!(
        report["counters"]["debug_inspection_nanos"]
            .as_u64()
            .expect("inspection timer")
            > 0
    );
    assert_eq!(report["debug"]["outcome"], "completed");
    assert!(
        report["wall_ms"].as_f64().expect("session wall time")
            - report["debug"]["resume_ms"].as_f64().expect("resume time")
            >= 200.0
    );
}
