//! Run server programs in disposable, independently supervised CLI processes.

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::super::process::Process;
use super::super::{create_temp_dir, run_cli, write_text};

const START: &str = "FPAS_SERVER_FIXTURE_BEGIN\n";
const READY: &str = "FPAS_SERVER_FIXTURE_PORT=";
const SOURCE: &str = "FPAS_SERVER_FIXTURE_SOURCE";

pub(super) struct Server {
    process: Option<Process>,
    cwd: PathBuf,
    port: u16,
}

impl Server {
    pub(super) fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Closing the process group precedes removing files still in use by it.
        drop(self.process.take());
        let _ = std::fs::remove_dir_all(&self.cwd);
    }
}

pub(super) fn start_server(name: &str, source: String) -> (PathBuf, Server) {
    let cwd = create_temp_dir(name);
    let server = spawn_server(&cwd, source);
    (cwd, server)
}

pub(super) fn spawn_server(cwd: &Path, source_text: String) -> Server {
    let mut server = Server {
        process: None,
        cwd: cwd.into(),
        port: 0,
    };
    let source = cwd.join("main.fpas");
    write_text(&source, &source_text);
    let mut command = Command::new(std::env::current_exe().expect("CLI test executable"));
    command
        .args([
            "--ignored",
            "--exact",
            "main_tests::network_server::fixture::server_child",
            "--nocapture",
        ])
        .env(SOURCE, &source)
        .current_dir(cwd);
    server.process =
        Some(Process::spawn(command, Duration::from_secs(30)).expect("start CLI server fixture"));
    let line = server
        .process
        .as_mut()
        .expect("server process")
        .wait_for_stdout_line(READY)
        .expect("CLI listener readiness");
    server.port = line
        .trim()
        .strip_prefix(READY)
        .expect("readiness prefix")
        .parse()
        .expect("OS-selected listener port");
    assert_ne!(server.port, 0, "listener must publish its assigned port");
    server
}

pub(super) fn connect_when_ready(port: u16) -> TcpStream {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(5))
        .expect("connect to ready FPAS listener");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("fixture read deadline");
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .expect("fixture write deadline");
    stream
}

pub(super) fn exchange(port: u16, fragments: &[&[u8]]) -> Vec<u8> {
    let mut stream = connect_when_ready(port);
    for fragment in fragments {
        stream.write_all(fragment).expect("write HTTP request");
    }
    stream
        .shutdown(Shutdown::Write)
        .expect("finish HTTP request");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read HTTP response");
    response
}

pub(super) fn finish_server(_cwd: impl AsRef<Path>, mut server: Server) -> (String, String) {
    let output = server
        .process
        .take()
        .expect("server process")
        .wait()
        .expect("CLI server deadline");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "CLI server failed: {stderr}");
    let captured = String::from_utf8(output.stdout).expect("CLI server UTF-8 output");
    let (_, program) = captured
        .split_once(START)
        .expect("CLI server output marker");
    let stdout = program
        .split_inclusive('\n')
        .filter(|line| !line.starts_with(READY))
        .collect::<String>();
    (stdout, stderr)
}

#[test]
#[ignore = "disposable CLI process owned by network server tests"]
fn server_child() {
    let source = PathBuf::from(std::env::var_os(SOURCE).expect("owned server source"));
    let cwd = source.parent().expect("server directory");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    print!("{START}");
    std::io::stdout().flush().expect("flush server marker");
    crate::cli_run::authorize_process_lifecycle();
    let exit = run_cli(
        &[
            "run".into(),
            "--std-lib".into(),
            root.join("lib").to_string_lossy().into_owned(),
            source.to_string_lossy().into_owned(),
        ],
        cwd,
        Box::new(std::io::stdout()),
        &mut std::io::stderr(),
        false,
    );
    std::io::stdout().flush().expect("flush program stdout");
    std::io::stderr().flush().expect("flush program stderr");
    std::process::exit(exit);
}
