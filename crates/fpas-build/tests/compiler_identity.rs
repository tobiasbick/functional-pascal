//! Integration tests for compiler identity emitted by the build script.

#![allow(
    clippy::expect_used,
    reason = "the build-script fixture has a required static declaration"
)]

const COMPILER_IDENTITY_SOURCE: &str = include_str!("../build/compiler_identity.rs");

#[path = "../build/compiler_identity.rs"]
mod compiler_identity;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Fixture(std::path::PathBuf);

impl Fixture {
    fn new(name: &str) -> std::io::Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "fpas-compiler-identity-{}-{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root)?;
        let fixture = Self(root);
        fixture.write("Cargo.toml", "[workspace]\n")?;
        fixture.write("Cargo.lock", "version = 4\n")?;
        let (_, declaration) = COMPILER_IDENTITY_SOURCE
            .split_once("const COMPILER_CRATES: &[&str] = &[")
            .expect("compiler crate declaration");
        let (list, _) = declaration.split_once("];").expect("compiler crate list");
        for name in list
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"'))
            .filter_map(|line| line.strip_suffix("\","))
        {
            fixture.write(&format!("crates/{name}/Cargo.toml"), name)?;
            fixture.write(&format!("crates/{name}/src/lib.rs"), "// source\n")?;
        }
        fixture.write("crates/fpas-build/build.rs", "// build entry\n")?;
        fixture.write(
            "crates/fpas-build/build/compiler_identity.rs",
            COMPILER_IDENTITY_SOURCE,
        )?;
        Ok(fixture)
    }

    fn write(&self, relative: &str, contents: &str) -> std::io::Result<()> {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("fixture parent"))?;
        std::fs::write(path, contents)
    }

    fn manifest(&self) -> std::path::PathBuf {
        self.0.join("crates/fpas-build")
    }

    fn identity(&self) -> std::io::Result<String> {
        compiler_identity::fingerprint(&self.manifest()).map(|(_, identity)| identity)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        assert!(self.0.starts_with(std::env::temp_dir()));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn ir_source_changes_and_new_nested_modules_invalidate_artifacts() -> TestResult {
    let fixture = Fixture::new("ir")?;
    let original = fixture.identity()?;
    assert_eq!(fixture.identity()?, original);
    fixture.write("crates/fpas-ir/src/lib.rs", "// changed folding\n")?;
    let changed = fixture.identity()?;
    assert_ne!(changed, original);
    fixture.write(
        "crates/fpas-ir/src/constants/checked.rs",
        "// checked operation\n",
    )?;
    let (sources, added) = compiler_identity::fingerprint(&fixture.manifest())?;
    assert_ne!(added, changed);
    assert!(sources.contains(&fixture.0.join("crates/fpas-ir/src/constants/checked.rs")));
    compiler_identity::emit(&fixture.manifest())?;
    Ok(())
}

#[test]
fn compiler_identity_is_independent_of_checkout_location() -> TestResult {
    let first = Fixture::new("first")?;
    let second = Fixture::new("second")?;
    assert_eq!(first.identity()?, second.identity()?);
    first.write("crates/fpas-ir/src/readme.txt", "not compiler source")?;
    assert_eq!(first.identity()?, second.identity()?);
    Ok(())
}

#[test]
fn compiler_identity_lists_every_build_relevant_workspace_crate() {
    let expected = [
        "fpas-build",
        "fpas-bytecode",
        "fpas-compiler",
        "fpas-ir",
        "fpas-lexer",
        "fpas-linker",
        "fpas-parser",
        "fpas-program",
        "fpas-project",
        "fpas-sema",
        "fpas-std",
        "fpas-unit",
    ];

    let (_, after_declaration) = COMPILER_IDENTITY_SOURCE
        .split_once("const COMPILER_CRATES: &[&str] = &[")
        .expect("compiler identity crate declaration");
    let (list, _) = after_declaration
        .split_once("];")
        .expect("compiler identity crate list terminator");
    let actual = list
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|line| line.strip_suffix("\","))
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}
