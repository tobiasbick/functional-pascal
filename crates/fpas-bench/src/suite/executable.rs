//! Locate the release FPAS executable through Cargo build artifacts.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    target: Option<CargoTarget>,
    executable: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
}

/// Build and return the release `fpas` executable.
pub fn ensure_release_fpas(repo_root: &Path) -> Result<PathBuf, String> {
    eprintln!("building fpas-cli --release…");
    let output = Command::new("cargo")
        .args([
            "build",
            "--release",
            "-p",
            "fpas-cli",
            "--message-format=json-render-diagnostics",
        ])
        .current_dir(repo_root)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo build --release -p fpas-cli failed ({})",
            output.status
        ));
    }
    let fpas = release_fpas_from_messages(&output.stdout)?;
    if !fpas.is_file() {
        return Err(format!(
            "cargo reported release binary at {}, but it is missing after build",
            fpas.display()
        ));
    }
    Ok(fpas)
}

fn release_fpas_from_messages(stdout: &[u8]) -> Result<PathBuf, String> {
    let stdout = std::str::from_utf8(stdout)
        .map_err(|error| format!("cargo build emitted non-UTF-8 JSON: {error}"))?;
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let message: CargoMessage = serde_json::from_str(line)
            .map_err(|error| format!("failed to parse cargo build message: {error}"))?;
        let Some(target) = message.target else {
            continue;
        };
        if message.reason == "compiler-artifact"
            && target.name == "fpas"
            && target.kind.iter().any(|kind| kind == "bin")
            && let Some(executable) = message.executable
        {
            return Ok(executable);
        }
    }
    Err("cargo build did not report the `fpas` executable artifact".to_owned())
}

#[cfg(test)]
mod tests {
    use super::ensure_release_fpas;
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::process::Command;
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
            format!(
                "[build]\ntarget-dir = \"custom-target\"\ntarget = \"{}\"\n",
                host_target()?
            ),
        )?;
        fs::write(
            workspace.join("fpas-cli/Cargo.toml"),
            "[package]\nname = \"fpas-cli\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"fpas\"\npath = \"src/main.rs\"\n",
        )?;
        fs::write(workspace.join("fpas-cli/src/main.rs"), "fn main() {}\n")?;
        let mut executable = workspace
            .join("custom-target")
            .join(host_target()?)
            .join("release/fpas");
        if cfg!(windows) {
            executable.set_extension("exe");
        }
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

    fn host_target() -> Result<String, Box<dyn Error>> {
        let output = Command::new("rustc").arg("-vV").output()?;
        let stdout = String::from_utf8(output.stdout)?;
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("host: ").map(str::to_owned))
            .ok_or_else(|| io::Error::other("rustc did not report its host target").into())
    }
}
