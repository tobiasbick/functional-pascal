//! Build-script entry points must resolve paths from the executing Cargo invocation.

use std::fs;
use std::io;
use std::process::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn check_entry_point(name: &str, source: &str, modules: &[(&str, &str)]) -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "fpas-build-script-environment-{}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    fs::write(root.join("build.rs"), source)?;
    for (path, contents) in modules {
        let path = root.join(path);
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("generated module path has no parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(path, contents)?;
    }
    let compiled_root = root.join("compile-checkout");
    let executing_root = root.join("executing-checkout");
    let binary = root.join(format!("build-script{}", std::env::consts::EXE_SUFFIX));
    let compilation = Command::new("rustc")
        .arg("--edition=2024")
        .arg(root.join("build.rs"))
        .arg("-o")
        .arg(&binary)
        .env("CARGO_MANIFEST_DIR", &compiled_root)
        .env("CARGO_PKG_VERSION", "0.0.1")
        .output()?;
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );

    let output = Command::new(&binary)
        .env("CARGO_MANIFEST_DIR", &executing_root)
        .env("OUT_DIR", root.join("target/debug/build/package/out"))
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout)?;
    let executing_root = executing_root
        .to_str()
        .ok_or_else(|| io::Error::other("executing checkout path is not UTF-8"))?;
    let compiled_root = compiled_root
        .to_str()
        .ok_or_else(|| io::Error::other("compiled checkout path is not UTF-8"))?;
    assert!(stdout.contains(executing_root), "{stdout}");
    assert!(!stdout.contains(compiled_root), "{stdout}");

    let missing = Command::new(&binary)
        .env_remove("CARGO_MANIFEST_DIR")
        .env("OUT_DIR", root.join("target/debug/build/package/out"))
        .output()?;
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("CARGO_MANIFEST_DIR"));
    assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn compiler_identity_entry_uses_the_executing_cargo_directory() -> TestResult {
    check_entry_point(
        "identity",
        include_str!("../build.rs"),
        &[(
            "build/compiler_identity.rs",
            r#"
pub fn emit(path: &std::path::Path) -> std::io::Result<()> {
    println!("{}", path.display());
    Ok(())
}
"#,
        )],
    )
}

#[test]
fn cli_entry_stages_from_the_executing_cargo_directory() -> TestResult {
    check_entry_point(
        "cli",
        include_str!("../../fpas-cli/build.rs"),
        &[
            (
                "stdlib_sync.rs",
                r#"
pub fn replace_tree(source: &std::path::Path, _: &std::path::Path) -> std::io::Result<()> {
    println!("{}", source.display());
    Ok(())
}
"#,
            ),
            (
                "version_sync.rs",
                r#"
pub fn validate_std_version(_: &std::path::Path, _: &str) -> std::io::Result<()> { Ok(()) }
"#,
            ),
        ],
    )
}
