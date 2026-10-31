use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[path = "identity_dependencies.rs"]
mod dependencies;

pub(crate) const COMPILER_CRATES: &[&str] = &[
    "fpas-acp-model",
    "fpas-binary",
    "fpas-build",
    "fpas-builtins",
    "fpas-code",
    "fpas-compiler",
    "fpas-diagnostics",
    "fpas-ir",
    "fpas-lexer",
    "fpas-linker",
    "fpas-numeric",
    "fpas-parser",
    "fpas-program",
    "fpas-project",
    "fpas-sema",
    "fpas-unit",
];

pub(crate) fn emit(manifest_dir: &Path) -> io::Result<()> {
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            io::Error::other("fpas-build must be below the workspace crates directory")
        })?;
    println!(
        "cargo:rustc-env=FPAS_COMPILER_BUILD_ID=source-{}",
        identity(workspace_root)?
    );
    Ok(())
}

pub(crate) fn identity(workspace_root: &Path) -> io::Result<String> {
    let manifest_dir = workspace_root.join("crates/fpas-build");
    let mut sources = Vec::new();

    for crate_name in COMPILER_CRATES {
        let crate_root = workspace_root.join("crates").join(crate_name);
        println!(
            "cargo:rerun-if-changed={}",
            crate_root.join("src").display()
        );
        collect_rust_sources(&crate_root.join("src"), &mut sources)?;
    }
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("build").display()
    );
    sources.push(manifest_dir.join("build.rs"));
    collect_rust_sources(&manifest_dir.join("build"), &mut sources)?;
    sources.sort();

    let mut hasher = blake3::Hasher::new();
    dependencies::hash(workspace_root, COMPILER_CRATES, &mut hasher)?;
    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        let relative = source.strip_prefix(workspace_root).map_err(|_| {
            io::Error::other("compiler identity source must be inside the workspace")
        })?;
        let relative = relative.to_string_lossy().replace('\\', "/");
        let test_path = relative
            .strip_prefix("crates/")
            .unwrap_or_default()
            .replace("/src/", "/");
        if TEST_SOURCES.iter().any(|test| {
            test_path == *test
                || test_path
                    .strip_prefix(test)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        }) {
            continue;
        }
        let bytes = fs::read(&source)?;
        hasher.update(&(relative.len() as u64).to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

// Explicit cfg(test) module roots, relative to each crate's src directory.
// Keep unknown/new source files in the identity; a filename alone never excludes code.
const TEST_SOURCES: &[&str] = &[
    "fpas-build/program_artifact/tests.rs",
    "fpas-build/engine/error/tests.rs",
    "fpas-code/intrinsic/tests.rs",
    "fpas-code/intrinsic/tests",
    "fpas-compiler/tests",
    "fpas-compiler/bytecode/allocation/tests.rs",
    "fpas-compiler/bytecode/allocation/tests",
    "fpas-compiler/intrinsic_catalog/tests.rs",
    "fpas-compiler/lowering/context/block_order_tests.rs",
    "fpas-diagnostics/codes/tests.rs",
    "fpas-lexer/tests",
    "fpas-numeric/tests",
    "fpas-parser/tests",
    "fpas-program/image/tests.rs",
    "fpas-project/standard_library/tests.rs",
    "fpas-project/source_inputs/tests.rs",
    "fpas-project/loading/tests.rs",
    "fpas-project/paths/exclusion_scaling.rs",
    "fpas-sema/tests",
    "fpas-sema/types/tests.rs",
    "fpas-sema/interface/tests.rs",
    "fpas-sema/interface/tests",
    "fpas-sema/scope/tests.rs",
    "fpas-sema/scope/capabilities/tests.rs",
    "fpas-sema/std_registry/native/tests.rs",
    "fpas-sema/std_registry/native/tests",
    "fpas-sema/std_registry/loaded/type_registration_tests.rs",
    "fpas-sema/check/decl/types/collection/finite/graph/tests.rs",
    "fpas-unit/object/debug_types/tests.rs",
];

fn collect_rust_sources(directory: &Path, sources: &mut Vec<PathBuf>) -> io::Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_rust_sources(&path, sources)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
    Ok(())
}
