//! Synchronizes generated standard-library metadata before building the CLI.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

mod stdlib_sync;
mod version_sync;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").ok_or("Cargo must set CARGO_MANIFEST_DIR")?,
    );
    // fpas-stdlib tracks its included sources, excluding generated sidecars.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=stdlib_sync.rs");
    println!("cargo:rerun-if-changed=version_sync.rs");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_BUILD_TARGET_DIR");
    let source_root = Path::new(fpas_stdlib::SOURCE_ROOT);
    version_sync::validate_std_version(
        &source_root.join("Std/Version.fpas"),
        env!("CARGO_PKG_VERSION"),
    )?;

    let out_dir = std::env::var_os("OUT_DIR")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo must set OUT_DIR"))?;
    let profile_dir = Path::new(&out_dir).ancestors().nth(3).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cargo OUT_DIR must be below the target profile directory",
        )
    })?;
    let output = Command::new(std::env::var_os("CARGO").ok_or("Cargo must set CARGO")?)
        .args(["metadata", "--offline", "--no-deps", "--format-version=1"])
        .current_dir(&manifest_dir)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "Cannot resolve Cargo output directories: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
        .into());
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let directory = |name| -> io::Result<PathBuf> {
        metadata[name]
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::other(format!("Cargo metadata must contain {name}")))
    };
    let build_dir = directory("build_directory")?;
    let target_dir = directory("target_directory")?;
    let relative_profile = profile_dir.strip_prefix(&build_dir)?;
    let delivered_profile = target_dir.join(relative_profile);
    println!(
        "cargo:rustc-env=FPAS_BUILD_PROFILE_DIR={}",
        delivered_profile.display()
    );
    println!(
        "cargo:rustc-env=FPAS_BUILD_LIBRARY_DIR={}",
        profile_dir.join("lib").display()
    );
    // Test executables live with intermediate artifacts; delivered tools live in target-dir.
    stdlib_sync::replace_tree(source_root, &profile_dir.join("lib"))?;
    if delivered_profile != profile_dir {
        stdlib_sync::replace_tree(source_root, &delivered_profile.join("lib"))?;
    }
    Ok(())
}
