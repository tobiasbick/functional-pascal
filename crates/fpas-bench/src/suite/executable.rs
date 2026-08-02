//! Locate the release FPAS executable through Cargo metadata.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    target_directory: PathBuf,
}

/// Return the release `fpas` executable, building it when it is absent.
pub fn ensure_release_fpas(repo_root: &Path) -> Result<PathBuf, String> {
    ensure_release_fpas_with_target_override(repo_root, None)
}

fn ensure_release_fpas_with_target_override(
    repo_root: &Path,
    target_dir_override: Option<&Path>,
) -> Result<PathBuf, String> {
    let target_dir = query_cargo_target_dir(repo_root, target_dir_override)?;
    let fpas = release_fpas_path(&target_dir);
    if fpas.is_file() {
        return Ok(fpas);
    }

    eprintln!("release fpas not found; building fpas-cli --release…");
    let mut command = Command::new("cargo");
    command
        .args(["build", "--release", "-p", "fpas-cli"])
        .current_dir(repo_root);
    if let Some(target_dir) = target_dir_override {
        command.env("CARGO_TARGET_DIR", target_dir);
    }
    let status = command
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;
    if !status.success() {
        return Err(format!(
            "cargo build --release -p fpas-cli failed ({status})"
        ));
    }
    if !fpas.is_file() {
        return Err(format!(
            "expected release binary at {} after build",
            fpas.display()
        ));
    }
    Ok(fpas)
}

fn query_cargo_target_dir(
    repo_root: &Path,
    target_dir_override: Option<&Path>,
) -> Result<PathBuf, String> {
    let mut command = Command::new("cargo");
    command
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root);
    if let Some(target_dir) = target_dir_override {
        command.env("CARGO_TARGET_DIR", target_dir);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed ({})\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse cargo metadata: {error}"))?;
    Ok(metadata.target_directory)
}

fn release_fpas_path(target_dir: &Path) -> PathBuf {
    let mut path = target_dir.join("release/fpas");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

#[cfg(test)]
mod tests {
    use super::{ensure_release_fpas_with_target_override, release_fpas_path};
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::path::Path;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn repo_root() -> Result<&'static Path, io::Error> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| io::Error::other("crate directory should have a workspace root"))
    }

    #[test]
    fn executable_discovery_honors_custom_target_directory() -> Result<(), Box<dyn Error>> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let target_dir = std::env::temp_dir().join(format!(
            "fpas-bench-custom-target-{}-{id}",
            std::process::id()
        ));
        let executable = release_fpas_path(&target_dir);
        let parent = executable
            .parent()
            .ok_or_else(|| io::Error::other("release executable should have a parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(&executable, [])?;
        let resolved = ensure_release_fpas_with_target_override(repo_root()?, Some(&target_dir));
        fs::remove_dir_all(&target_dir)?;

        assert_eq!(resolved, Ok(executable));
        Ok(())
    }
}
