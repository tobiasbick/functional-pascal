//! Independent child and descendant used by pool and allocation regressions.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

fn setting(name: &str) -> String {
    std::env::args()
        .find_map(|arg| arg.strip_prefix(name).map(str::to_owned))
        .expect("child fixture metadata")
}

pub fn descendants(mode: &str) -> bool {
    matches!(
        mode,
        "descendants" | "cancel" | "scope_exit" | "vm_shutdown"
    )
}

#[expect(
    clippy::zombie_processes,
    reason = "The tested process-group owner must terminate the deliberately orphaned descendant"
)]
pub fn run() {
    // Spare skip filters carry fixture metadata through libtest's CLI. Their '='
    // cannot match a Rust test identity, so they never change test selection.
    let mode = setting("fpas-mode=");
    let port = setting("fpas-gate=");
    let observe = setting("fpas-observe=");
    let child_name = setting("fpas-child=");
    if descendants(&mode) {
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                &child_name,
                "--nocapture",
                "--skip",
                "fpas-mode=descendant",
                "--skip",
                &format!("fpas-gate={port}"),
                "--skip",
                &format!("fpas-observe={observe}"),
                "--skip",
                &format!("fpas-child={child_name}"),
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
    }
    let _observer = TcpStream::connect(format!("127.0.0.1:{observe}")).unwrap();
    let mut gate = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    gate.set_read_timeout(Some(Duration::from_secs(30)))
        .unwrap();
    if mode != "descendant" {
        std::io::stderr().write_all(b"prefix\n").unwrap();
    }
    gate.write_all(&[if mode == "descendant" { 2 } else { 1 }])
        .unwrap();
    let mut byte = [0];
    let released = gate.read(&mut byte).unwrap_or(0) == 1;
    if mode == "descendant" || !released {
        loop {
            std::thread::park();
        }
    }
    if mode == "select" {
        std::io::stdout().write_all(&vec![0; 65536]).unwrap();
        std::io::stderr().write_all(&vec![255; 65536]).unwrap();
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input).unwrap();
        assert_eq!(input.len(), 131072);
        assert!(input.iter().all(|byte| *byte == 105));
        std::io::stdout().write_all(&input).unwrap();
    } else if mode == "capture_options" {
        std::io::stdout().write_all(&vec![b'o'; 131072]).unwrap();
        std::io::stderr().write_all(&vec![b'e'; 131072]).unwrap();
        let mut input = Vec::new();
        std::io::stdin().read_to_end(&mut input).unwrap();
        assert_eq!(input.len(), 262144);
        assert!(input.iter().all(|byte| *byte == b'i'));
        std::io::stdout().write_all(&input).unwrap();
    } else if mode == "limit" {
        std::io::stderr().write_all(&vec![b'e'; 8192]).unwrap();
    }
    std::process::exit(7);
}
