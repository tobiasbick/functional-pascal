//! Process shutdown must not depend on a client replying to capability registration.
//!
//! **Documentation:** `docs/pascal/tools/editor-integration.md`

use std::process::Stdio;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdout, Command};

use super::support::{TempDirectory, exit, frame, initialize_with_root, initialized, shutdown};

#[tokio::test]
async fn shutdown_exits_with_an_unanswered_capability_registration() {
    let temp = TempDirectory::new("shutdown-pending-registration");
    let mut child = Command::new(env!("CARGO_BIN_EXE_fpas-lsp"))
        .current_dir(temp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .expect("start language server");
    let mut input = child.stdin.take().expect("stdin");
    let mut output = BufReader::new(child.stdout.take().expect("stdout"));
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut initialize = initialize_with_root(1, Some(&temp.uri(".")));
        initialize["params"]["capabilities"]["workspace"]["didChangeWatchedFiles"] =
            json!({"dynamicRegistration": true});
        input
            .write_all(&frame(&initialize))
            .await
            .expect("initialize");
        assert_eq!(read_message(&mut output).await["id"], json!(1));
        input
            .write_all(&frame(&initialized()))
            .await
            .expect("initialized");
        let registration = read_message(&mut output).await;
        assert_eq!(registration["method"], json!("client/registerCapability"));
        assert!(registration["id"].is_number());

        input
            .write_all(&frame(&shutdown(2)))
            .await
            .expect("shutdown");
        assert_eq!(read_message(&mut output).await["id"], json!(2));
        input.write_all(&frame(&exit())).await.expect("exit");
        assert!(child.wait().await.expect("server exit status").success());
    })
    .await
    .expect("shutdown and exit must finish without a registration reply");
}

async fn read_message(output: &mut BufReader<ChildStdout>) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        assert_ne!(output.read_line(&mut line).await.expect("LSP header"), 0);
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(value.trim().parse::<usize>().expect("content length"));
        }
    }
    let mut body = vec![0; content_length.expect("Content-Length")];
    output.read_exact(&mut body).await.expect("LSP body");
    serde_json::from_slice(&body).expect("JSON response")
}
