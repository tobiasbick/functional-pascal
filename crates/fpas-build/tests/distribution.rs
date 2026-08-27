//! Integration tests for assembling redistributable application directories.

#![allow(
    clippy::expect_used,
    reason = "distribution filesystem fixtures use direct assertions for diagnostic clarity"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use fpas_build::{BuildOptions, stage_standard_library};

fn temp_dir() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fpas-standard-library-distribution-{}-{id}",
        std::process::id()
    ))
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory must be created");
    }
    fs::write(path, contents).expect("fixture file must be written");
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    let output = std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}

#[cfg(unix)]
fn remove_directory_link(link: &Path) -> std::io::Result<()> {
    fs::remove_file(link)
}

#[cfg(windows)]
fn remove_directory_link(link: &Path) -> std::io::Result<()> {
    fs::remove_dir(link)
}

#[test]
fn staging_recompiles_every_unit_and_exactly_replaces_the_distribution() {
    let root = temp_dir();
    let staging = root.join("staging");
    let distribution = root.join("distribution");
    write(
        &staging.join("stdlib.fpasprj"),
        r#"[project]
name = "test-standard-library"
kind = "library"

[exports]
units = ["Std.DistributionFixture"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write(
        &staging.join("Std/DistributionFixture.fpas"),
        "unit Std.DistributionFixture;\nconst Answer: integer := 42;\n",
    );
    write(
        &staging.join("Std/DistributionFixture.fpascu"),
        "corrupt stale sidecar",
    );
    write(&staging.join("Std/Removed.fpascu"), "orphan");
    write(&staging.join("Std/Removed.fpascu.lock"), "orphan lock");
    write(&distribution.join("Std/Removed.fpas"), "stale source");

    let counters = stage_standard_library(&staging, &distribution, &BuildOptions::default())
        .expect("distribution staging must succeed");

    assert_eq!(counters.compiled, 1);
    assert_eq!(counters.sidecar_reused, 0);
    assert!(distribution.join("stdlib.fpasprj").is_file());
    assert!(distribution.join("Std/DistributionFixture.fpas").is_file());
    assert!(
        distribution
            .join("Std/DistributionFixture.fpascu")
            .is_file()
    );
    assert!(!distribution.join("Std/Removed.fpas").exists());
    assert!(!distribution.join("Std/Removed.fpascu").exists());
    assert!(!distribution.join("Std/Removed.fpascu.lock").exists());
    assert!(!staging.join("Std/Removed.fpascu").exists());
    assert!(!staging.join("Std/Removed.fpascu.lock").exists());
    assert!(
        fs::read_dir(&root)
            .expect("distribution parent must remain readable")
            .map(|entry| entry.expect("distribution parent entry").file_name())
            .all(|name| !name.to_string_lossy().contains("fpas-distribution-"))
    );
    fs::remove_dir_all(root).expect("temp directory must be removed");
}

#[test]
fn staging_rejects_the_same_source_and_destination_directory() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("fixture directory must be created");

    let error = stage_standard_library(&root, &root, &BuildOptions::default())
        .expect_err("identical directories must be rejected");

    assert!(error.to_string().contains("separate"));
    fs::remove_dir_all(root).expect("temp directory must be removed");
}

#[test]
fn staging_rejects_nested_distribution_directories() {
    let root = temp_dir();
    let staging = root.join("staging");
    fs::create_dir_all(&staging).expect("fixture directory must be created");

    let error = stage_standard_library(
        &staging,
        &staging.join("distribution"),
        &BuildOptions::default(),
    )
    .expect_err("nested directories must be rejected");

    assert!(error.to_string().contains("non-nested"));
    fs::remove_dir_all(root).expect("temp directory must be removed");
}

#[test]
fn failed_compilation_preserves_the_existing_distribution() {
    let root = temp_dir();
    let staging = root.join("staging");
    let distribution = root.join("distribution");
    write(
        &staging.join("stdlib.fpasprj"),
        r#"[project]
name = "invalid-standard-library"
kind = "library"

[exports]
units = ["Std.InvalidDistributionFixture"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write(
        &staging.join("Std/InvalidDistributionFixture.fpas"),
        "unit Std.InvalidDistributionFixture;\nconst Answer: integer := 'wrong';\n",
    );
    write(&distribution.join("marker.txt"), "previous distribution");

    stage_standard_library(&staging, &distribution, &BuildOptions::default())
        .expect_err("invalid source must fail distribution staging");

    assert_eq!(
        fs::read_to_string(distribution.join("marker.txt"))
            .expect("previous distribution must remain"),
        "previous distribution"
    );
    fs::remove_dir_all(root).expect("temp directory must be removed");
}

#[test]
fn staging_rejects_external_directory_links_without_cleaning_target() {
    let root = temp_dir();
    let outside = temp_dir();
    let staging = root.join("staging");
    let distribution = root.join("distribution");
    let victim = outside.join("External.fpascu");
    fs::create_dir_all(&staging).expect("staging directory must be created");
    write(&victim, "external sidecar");
    let link = staging.join("linked");
    create_directory_link(&outside, &link).expect("directory link fixture must be created");

    let result = stage_standard_library(&staging, &distribution, &BuildOptions::default());
    let victim_contents = fs::read_to_string(&victim).ok();

    remove_directory_link(&link).expect("directory symlink must be removed");
    fs::remove_dir_all(root).expect("temp directory must be removed");
    fs::remove_dir_all(outside).expect("outside directory must be removed");

    let error = result.expect_err("directory symlink must be rejected");
    assert_eq!(
        (
            error.to_string().contains("symbolic link"),
            victim_contents.as_deref()
        ),
        (true, Some("external sidecar"))
    );
}

#[test]
fn staging_rejects_cyclic_directory_links() {
    let root = temp_dir();
    let staging = root.join("staging");
    let distribution = root.join("distribution");
    fs::create_dir_all(&staging).expect("staging directory must be created");
    let link = staging.join("cycle");
    create_directory_link(&staging, &link).expect("cyclic directory link must be created");

    let result = stage_standard_library(&staging, &distribution, &BuildOptions::default());

    remove_directory_link(&link).expect("cyclic directory link must be removed");
    fs::remove_dir_all(root).expect("temp directory must be removed");

    let error = result.expect_err("cyclic directory link must be rejected");
    assert!(error.to_string().contains("symbolic link"));
}
