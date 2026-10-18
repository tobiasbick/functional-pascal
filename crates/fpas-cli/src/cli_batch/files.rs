//! Runner-owned request, cancellation and atomic result files.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Serialize, Deserialize)]
pub(super) struct Request {
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

pub(super) struct Files {
    pub root: PathBuf,
    owned: bool,
}
impl Files {
    pub fn create() -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        loop {
            let root = std::env::temp_dir().join(format!(
                "fpas-debug-batch-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&root) {
                Ok(()) => return Ok(Self { root, owned: true }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
    pub fn attach(path: &Path) -> Self {
        Self {
            root: path.into(),
            owned: false,
        }
    }
    pub fn cancel(&self) -> PathBuf {
        self.root.join("cancel")
    }
    pub fn result(&self) -> PathBuf {
        self.root.join("result.json")
    }
    pub fn write_request(&self, request: &Request) -> io::Result<()> {
        fs::write(self.root.join("request.json"), serde_json::to_vec(request)?)
    }
    pub fn read_request(&self) -> io::Result<Request> {
        // This private argv envelope is separate from scripted input. JSON escaping
        // can expand valid OS arguments well beyond the script's 1 MiB ceiling.
        serde_json::from_slice(&read_bounded(
            &self.root.join("request.json"),
            64 * 1024 * 1024,
        )?)
        .map_err(io::Error::other)
    }
    pub fn read_result(&self) -> Result<serde_json::Value, String> {
        let bytes = read_bounded(&self.result(), fpas_debug::batch::MAX_REPORT_BYTES)
            .map_err(|error| error.to_string())?;
        let report: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if report["schema_version"] != 1
            || !report["capture"]["reasons"].is_array()
            || !report["diagnostics"].is_array()
            || !report["exit_code"].as_u64().is_some_and(|exit| exit <= 3)
        {
            return Err("Worker published an invalid result envelope.".into());
        }
        Ok(report)
    }
    pub fn publish(&self, report: &mut serde_json::Value) -> io::Result<()> {
        fs::write(
            self.root.join("result.tmp"),
            fpas_debug::batch::finish_report(report),
        )?;
        fs::rename(self.root.join("result.tmp"), self.result())
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        if self.owned {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

pub(super) fn read_bounded(path: &Path, maximum: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(io::Error::other("File exceeds its byte limit."));
    }
    Ok(bytes)
}
