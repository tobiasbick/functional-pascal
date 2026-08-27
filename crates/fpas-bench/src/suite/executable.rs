//! Locate the release FPAS executable through Cargo metadata.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    target_directory: PathBuf,
}

/// Build and return the release `fpas` executable.
pub fn ensure_release_fpas(repo_root: &Path) -> Result<PathBuf, String> {
    let target_dir = query_cargo_target_dir(repo_root)?;
    let fpas = release_fpas_path(&target_dir);

    eprintln!("building fpas-cli --release…");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "fpas-cli"])
        .current_dir(repo_root)
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

fn query_cargo_target_dir(repo_root: &Path) -> Result<PathBuf, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root)
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
    use super::{ensure_release_fpas, release_fpas_path};
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn existing_release_executable_is_rebuilt_in_configured_target_directory()
    -> Result<(), Box<dyn Error>> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let workspace = std::env::temp_dir().join(format!(
            "fpas-bench-rebuild-existing-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(workspace.join("fpas-cli/src"))?;
        fs::create_dir_all(workspace.join(".cargo"))?;
        fs::write(
            workspace.join("Cargo.toml"),
            "[workspace]\nresolver = \"3\"\nmembers = [\"fpas-cli\"]\n",
        )?;
        fs::write(
            workspace.join(".cargo/config.toml"),
            "[build]\ntarget-dir = \"custom-target\"\n",
        )?;
        fs::write(
            workspace.join("fpas-cli/Cargo.toml"),
            "[package]\nname = \"fpas-cli\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"fpas\"\npath = \"src/main.rs\"\n",
        )?;
        fs::write(workspace.join("fpas-cli/src/main.rs"), "fn main() {}\n")?;
        let executable = release_fpas_path(&workspace.join("custom-target"));
        let parent = executable
            .parent()
            .ok_or_else(|| io::Error::other("release executable should have a parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(&executable, [])?;

        let resolved = ensure_release_fpas(&workspace);
        let executable_len = fs::metadata(&executable)?.len();
        fs::remove_dir_all(&workspace)?;

        assert_eq!(resolved, Ok(executable));
        assert!(
            executable_len > 0,
            "Cargo must replace the stale executable"
        );
        Ok(())
    }
}
