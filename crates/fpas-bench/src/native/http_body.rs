//! Local HTTP fixture for the buffered-body execution benchmark.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

pub(super) fn run(iterations: usize, fragment_bytes: usize) -> Result<(), String> {
    if !(1..=65536).contains(&fragment_bytes) {
        return Err("HTTP fragment size must be in 1..=65536".to_string());
    }
    let fpas = std::env::var_os("FPAS_BENCH_CLI")
        .ok_or("Run this workload through cargo bench-fpas run --group review")?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let stopped = Arc::new(AtomicBool::new(false));
    let server_stopped = Arc::clone(&stopped);
    let server = std::thread::spawn(move || -> Result<(), String> {
        let fragment = vec![b'A'; fragment_bytes];
        while !server_stopped.load(Ordering::Acquire) {
            let mut stream = match listener.accept() {
                Ok((stream, _)) => stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(error) => return Err(error.to_string()),
            };
            stream
                .set_nonblocking(false)
                .map_err(|error| error.to_string())?;
            stream
                .set_nodelay(true)
                .map_err(|error| error.to_string())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| error.to_string())?;
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .map_err(|error| error.to_string())?;
            let mut head = Vec::new();
            while !head.windows(4).any(|part| part == b"\r\n\r\n") {
                let mut buffer = [0; 1024];
                let count = stream
                    .read(&mut buffer)
                    .map_err(|error| error.to_string())?;
                if count == 0 || head.len() > 65536 {
                    return Err("Invalid benchmark request".to_string());
                }
                head.extend_from_slice(&buffer[..count]);
            }
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                fragment_bytes * 128
            )
            .map_err(|error| error.to_string())?;
            for _ in 0..128 {
                stream
                    .write_all(&fragment)
                    .map_err(|error| error.to_string())?;
            }
            stream
                .shutdown(std::net::Shutdown::Write)
                .map_err(|error| error.to_string())?;
            let mut tail = [0; 1024];
            while stream.read(&mut tail).map_err(|error| error.to_string())? != 0 {}
        }
        Ok(())
    });
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("Missing benchmark workspace")?;
    let output = Command::new(fpas)
        .arg("run")
        .arg(root.join("examples/pascal/network/http_body_accumulation_benchmark.fpas"))
        .arg("--")
        .arg(iterations.to_string())
        .arg(fragment_bytes.to_string())
        .arg(format!("http://127.0.0.1:{port}/"))
        .output();
    stopped.store(true, Ordering::Release);
    let server_result = server.join().map_err(|_| "HTTP fixture panicked")?;
    let output = output.map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{}\nfixture: {server_result:?}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    server_result?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
