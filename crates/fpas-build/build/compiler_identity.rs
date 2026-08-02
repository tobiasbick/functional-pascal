use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const COMPILER_CRATES: &[&str] = &[
    "fpas-build",
    "fpas-bytecode",
    "fpas-compiler",
    "fpas-lexer",
    "fpas-linker",
    "fpas-parser",
    "fpas-program",
    "fpas-project",
    "fpas-sema",
    "fpas-std",
    "fpas-unit",
];

pub(crate) fn emit(manifest_dir: &Path) -> io::Result<()> {
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            io::Error::other("fpas-build must be below the workspace crates directory")
        })?;
    let mut sources = vec![
        workspace_root.join("Cargo.lock"),
        workspace_root.join("Cargo.toml"),
    ];

    for crate_name in COMPILER_CRATES {
        let crate_root = workspace_root.join("crates").join(crate_name);
        println!(
            "cargo:rerun-if-changed={}",
            crate_root.join("src").display()
        );
        sources.push(crate_root.join("Cargo.toml"));
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
    for source in sources {
        println!("cargo:rerun-if-changed={}", source.display());
        let relative = source.strip_prefix(workspace_root).map_err(|_| {
            io::Error::other("compiler identity source must be inside the workspace")
        })?;
        let relative = relative.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(&source)?;
        hasher.update(&(relative.len() as u64).to_le_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }

    println!(
        "cargo:rustc-env=FPAS_COMPILER_BUILD_ID=source-{}",
        hasher.finalize().to_hex()
    );
    Ok(())
}

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
