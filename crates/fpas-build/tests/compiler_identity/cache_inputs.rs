//! Artifact freshness depends on compiler inputs, not unrelated workspace changes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::identity;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> io::Result<Self> {
        let root =
            std::env::temp_dir().join(format!("fpas-identity-inputs-{}", std::process::id()));
        let fixture = Self(root);
        fixture.write("Cargo.toml", WORKSPACE)?;
        let mut lock = String::from("version = 4\n");
        for name in identity::COMPILER_CRATES {
            fixture.write(&format!("crates/{name}/src/lib.rs"), "// compiler source\n")?;
            let mut manifest = format!("[package]\nname = '{name}'\nversion = '0.0.1'\n");
            lock.push_str(&format!(
                "[[package]]\nname = '{name}'\nversion = '0.0.1'\n"
            ));
            if *name == "fpas-build" {
                manifest.push_str("[dependencies]\ncompiler-dep.workspace = true\n[dev-dependencies]\nruntime-dep.workspace = true\n");
                lock.push_str("dependencies = ['compiler-dep', 'runtime-dep']\n");
            }
            fixture.write(&format!("crates/{name}/Cargo.toml"), &manifest)?;
        }
        fixture.write("crates/fpas-build/build.rs", "// build identity entry\n")?;
        fixture.write(
            "crates/fpas-compiler/src/tests/cases.rs",
            "// test source\n",
        )?;
        lock.push_str(DEPENDENCIES);
        fixture.write("Cargo.lock", &lock)?;
        Ok(fixture)
    }

    fn write(&self, path: &str, text: &str) -> io::Result<()> {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
        fs::write(path, text)
    }

    fn replace(&self, path: &str, old: &str, new: &str) -> io::Result<()> {
        let text = fs::read_to_string(self.0.join(path))?;
        assert!(text.contains(old));
        self.write(path, &text.replace(old, new))
    }

    fn id(&self) -> io::Result<String> {
        identity::identity(&self.0)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn identity_ignores_test_and_runtime_edits_but_tracks_compiler_dependencies() -> io::Result<()> {
    let fixture = Fixture::new()?;
    let original = fixture.id()?;
    fixture.write(
        "crates/fpas-compiler/src/tests/cases.rs",
        "// changed test\n",
    )?;
    fixture.replace("Cargo.toml", "runtime-dep = '1'", "runtime-dep = '2'")?;
    fixture.replace(
        "Cargo.lock",
        "name = 'runtime-dep'\nversion = '1.0.0'",
        "name = 'runtime-dep'\nversion = '2.0.0'",
    )?;
    assert_eq!(
        original,
        fixture.id()?,
        "runtime-only upgrades and tests keep artifacts fresh"
    );

    fixture.write(
        "crates/fpas-compiler/src/new_pass.rs",
        "// new production source\n",
    )?;
    assert_ne!(
        original,
        fixture.id()?,
        "new compiler files invalidate artifacts"
    );
    fs::remove_file(fixture.0.join("crates/fpas-compiler/src/new_pass.rs"))?;
    assert_eq!(original, fixture.id()?);

    fixture.replace(
        "Cargo.lock",
        "checksum = 'compiler-a'",
        "checksum = 'compiler-b'",
    )?;
    let compiler_upgrade = fixture.id()?;
    assert_ne!(
        original, compiler_upgrade,
        "compiler dependency content invalidates artifacts"
    );
    fixture.replace(
        "Cargo.lock",
        "checksum = 'shared-a'",
        "checksum = 'shared-b'",
    )?;
    assert_ne!(
        compiler_upgrade,
        fixture.id()?,
        "transitive shared dependencies still invalidate artifacts"
    );
    let before_features = fixture.id()?;
    fixture.replace(
        "Cargo.toml",
        "compiler-dep = '1'",
        "compiler-dep = { version = '1', features = ['extra'] }",
    )?;
    assert_ne!(
        before_features,
        fixture.id()?,
        "compiler feature changes invalidate artifacts"
    );
    Ok(())
}

const WORKSPACE: &str = r#"
[workspace]
resolver = '3'
[workspace.dependencies]
compiler-dep = '1'
runtime-dep = '1'
"#;

const DEPENDENCIES: &str = r#"
[[package]]
name = 'compiler-dep'
version = '1.0.0'
checksum = 'compiler-a'
dependencies = ['shared-dep 1.0.0']
[[package]]
name = 'runtime-dep'
version = '1.0.0'
dependencies = ['shared-dep 2.0.0']
[[package]]
name = 'shared-dep'
version = '1.0.0'
checksum = 'shared-a'
[[package]]
name = 'shared-dep'
version = '2.0.0'
checksum = 'runtime-shared-a'
"#;
