//! Build-script entry points must resolve paths from the executing Cargo invocation.

use std::fs;
use std::process::Command;

fn check_entry_point(name: &str, source: &str, modules: &[(&str, &str)]) {
    let root = std::env::temp_dir().join(format!(
        "fpas-build-script-environment-{}-{name}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("build.rs"), source).unwrap();
    for (path, contents) in modules {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
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
        .output()
        .unwrap();
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );

    let output = Command::new(&binary)
        .env("CARGO_MANIFEST_DIR", &executing_root)
        .env("OUT_DIR", root.join("target/debug/build/package/out"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(executing_root.to_str().unwrap()),
        "{stdout}"
    );
    assert!(
        !stdout.contains(compiled_root.to_str().unwrap()),
        "{stdout}"
    );

    let missing = Command::new(&binary)
        .env_remove("CARGO_MANIFEST_DIR")
        .env("OUT_DIR", root.join("target/debug/build/package/out"))
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("CARGO_MANIFEST_DIR"));
    assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn compiler_identity_entry_uses_the_executing_cargo_directory() {
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
    );
}

#[test]
fn cli_entry_stages_from_the_executing_cargo_directory() {
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
    );
}
