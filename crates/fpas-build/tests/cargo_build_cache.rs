//! Exercise Cargo freshness and CLI library delivery with real, small workspaces.

use std::fs::{self, File, FileTimes};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> std::io::Result<Self> {
        let root =
            std::env::temp_dir().join(format!("fpas-cargo-cache-{}-{name}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(Self(root))
    }

    fn checkout(&self, name: &str, value: &str) -> TestResult {
        let root = self.0.join(name);
        write(
            &root,
            "rust-toolchain.toml",
            include_str!("../../../rust-toolchain.toml"),
        )?;
        write(
            &root,
            ".cargo/config.toml",
            include_str!("../../../.cargo/config.toml"),
        )?;
        write(
            &root,
            "Cargo.toml",
            r#"
[workspace]
members = ["crates/fpas-cli", "crates/value", "lib"]
resolver = "3"
[workspace.package]
version = "0.0.1"
edition = "2024"
rust-version = "1.99.1"
license = "BSD-3-Clause"
repository = "https://example.invalid/fpas"
[workspace.lints.rust]
unsafe_code = "forbid"
"#,
        )?;
        write(
            &root,
            "lib/Cargo.toml",
            include_str!("../../../lib/Cargo.toml"),
        )?;
        write(&root, "lib/cargo.rs", include_str!("../../../lib/cargo.rs"))?;
        write(&root, "lib/build.rs", include_str!("../../../lib/build.rs"))?;
        write(
            &root,
            "lib/stdlib.fpasprj",
            "[project]\nname = 'stdlib'\nkind = 'library'\n",
        )?;
        write(
            &root,
            "lib/Std/Version.fpas",
            &format!(
                "public const CompilerVersion: string := '0.0.1';\npublic const LibraryVersion: string := '0.0.1';\n// {value}\n"
            ),
        )?;
        write(
            &root,
            "crates/value/Cargo.toml",
            "[package]\nname = 'value'\nversion = '0.0.1'\nedition = '2024'\n",
        )?;
        write(
            &root,
            "crates/value/src/lib.rs",
            &format!("pub const VALUE: &str = {value:?};\n"),
        )?;
        write(
            &root,
            "crates/fpas-cli/Cargo.toml",
            r#"
[package]
name = "fpas-cli"
version = "0.0.1"
edition = "2024"
[[bin]]
name = "fpas"
path = "src/main.rs"
[dependencies]
value = { path = "../value" }
[build-dependencies]
fpas-stdlib = { path = "../../lib" }
serde_json = "1"
"#,
        )?;
        write(
            &root,
            "crates/fpas-cli/src/main.rs",
            r#"
mod location;
fn main() {
    let library = location::resolve(
        &std::env::current_exe().unwrap(),
        std::path::Path::new(env!("FPAS_BUILD_PROFILE_DIR")),
        std::path::Path::new(env!("FPAS_BUILD_LIBRARY_DIR")),
    ).unwrap();
    let version = std::fs::read_to_string(library.join("Std/Version.fpas")).unwrap();
    println!("{}|{}", value::VALUE, version.lines().last().unwrap());
}
"#,
        )?;
        write(
            &root,
            "crates/fpas-cli/src/location.rs",
            include_str!("../../fpas-cli/src/standard_library/location.rs"),
        )?;
        write(
            &root,
            "crates/fpas-cli/build.rs",
            include_str!("../../fpas-cli/build.rs"),
        )?;
        write(
            &root,
            "crates/fpas-cli/stdlib_sync.rs",
            include_str!("../../fpas-cli/stdlib_sync.rs"),
        )?;
        write(
            &root,
            "crates/fpas-cli/version_sync.rs",
            include_str!("../../fpas-cli/version_sync.rs"),
        )?;
        // A restored checkout can have older source timestamps than shared build artifacts.
        File::options()
            .write(true)
            .open(root.join("crates/value/src/lib.rs"))?
            .set_times(
                FileTimes::new().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(86400)),
            )?;
        Ok(())
    }

    fn build(&self, checkout: &str) -> Result<bool, Box<dyn std::error::Error>> {
        self.build_with_target_dir(checkout, None)
    }

    fn build_with_target_dir(
        &self,
        checkout: &str,
        target_dir: Option<&Path>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut command = Command::new("cargo");
        command.args([
            "build",
            "--offline",
            "-p",
            "fpas-cli",
            "--message-format=json",
        ]);
        if let Some(target_dir) = target_dir {
            command.arg("--target-dir").arg(target_dir);
        }
        let output = command
            .current_dir(self.0.join(checkout))
            .env("CARGO_TARGET_DIR", self.0.join("target"))
            .env_remove("CARGO_BUILD_BUILD_DIR")
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout)?;
        let artifact = stdout
            .lines()
            .find(|line| {
                line.contains("\"reason\":\"compiler-artifact\"")
                    && line.contains("\"name\":\"fpas\"")
            })
            .ok_or("Cargo did not report the CLI artifact")?;
        Ok(artifact.contains("\"fresh\":true"))
    }

    fn output(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new(
            self.0
                .join("target/debug")
                .join(format!("fpas{}", std::env::consts::EXE_SUFFIX)),
        )
        .output()?;
        assert!(output.status.success());
        Ok(String::from_utf8(output.stdout)?.trim().to_owned())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(root: &Path, path: &str, text: &str) -> std::io::Result<()> {
    let path = root.join(path);
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| std::io::Error::other("missing parent"))?,
    )?;
    fs::write(path, text)
}

#[test]
fn shared_target_directory_keeps_each_checkouts_compiler_and_library() -> TestResult {
    let fixture = Fixture::new("checkouts")?;
    fixture.checkout("before", "before")?;
    fixture.checkout("after", "after")?;
    for checkout in ["before", "after", "before"] {
        fixture.build(checkout)?;
        assert_eq!(
            fixture.output()?,
            format!("{checkout}|// {checkout}"),
            "compiler belongs to the wrong checkout"
        );
        assert!(
            fixture
                .0
                .join("target/debug/lib/Std/Version.fpas")
                .is_file()
        );
    }
    let explicit_target = fixture.0.join("explicit-target");
    write(
        &explicit_target,
        "debug/lib/stdlib.fpasprj",
        "older library",
    )?;
    write(
        &explicit_target,
        "debug/lib/Std/Version.fpas",
        "// older library",
    )?;
    fixture.build_with_target_dir("before", Some(&explicit_target))?;
    let executable = format!("fpas{}", std::env::consts::EXE_SUFFIX);
    let output = Command::new(explicit_target.join("debug").join(&executable)).output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?.trim(), "before|// before");
    let installed = fixture.0.join("installed");
    fs::create_dir_all(&installed)?;
    let executable = format!("fpas{}", std::env::consts::EXE_SUFFIX);
    fs::copy(
        fixture.0.join("target/debug").join(&executable),
        installed.join(&executable),
    )?;
    write(&installed, "lib/stdlib.fpasprj", "installed library")?;
    write(&installed, "lib/Std/Version.fpas", "// installed")?;
    let output = Command::new(installed.join(executable)).output()?;
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)?.trim(),
        "before|// installed"
    );
    Ok(())
}

#[test]
fn library_sidecars_stay_fresh_while_source_changes_are_delivered() -> TestResult {
    let fixture = Fixture::new("sidecars")?;
    fixture.checkout("source", "current")?;
    assert!(!fixture.build("source")?);
    assert!(fixture.build("source")?);
    let source = fixture.0.join("source");
    for name in [
        "Version.fpascu",
        "Version.fpascu.lock",
        "Version.fpascu.tmp-1",
    ] {
        write(&source, &format!("lib/Std/{name}"), "derived")?;
        assert!(fixture.build("source")?, "creating {name} rebuilt the CLI");
        write(&source, &format!("lib/Std/{name}"), "changed")?;
        assert!(fixture.build("source")?, "editing {name} rebuilt the CLI");
        fs::remove_file(source.join("lib/Std").join(name))?;
        assert!(fixture.build("source")?, "removing {name} rebuilt the CLI");
    }
    write(&source, "lib/Std/Added.fpas", "new source")?;
    assert!(!fixture.build("source")?);
    let delivered = fixture.0.join("target/debug/lib/Std/Added.fpas");
    assert_eq!(fs::read_to_string(&delivered)?, "new source");
    write(&source, "lib/Std/Added.fpas", "changed source")?;
    assert!(!fixture.build("source")?);
    assert_eq!(fs::read_to_string(&delivered)?, "changed source");
    fs::remove_file(source.join("lib/Std/Added.fpas"))?;
    assert!(!fixture.build("source")?);
    assert!(!delivered.exists());
    assert!(fixture.build("source")?);
    Ok(())
}
