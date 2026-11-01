//! The semantic fingerprint excludes adapter code and absolute checkout paths.

use crate::encoding::CanonicalHasher;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

pub(crate) fn emit(manifest_dir: &Path) -> io::Result<()> {
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("Analysis crate must be in the workspace."))?;
    let mut files = BTreeSet::from([root.join("Cargo.toml"), root.join("Cargo.lock")]);
    let workspace = manifest(&root.join("Cargo.toml"))?;
    let shared = workspace
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let mut visited = BTreeSet::new();
    collect_package(root, manifest_dir, &shared, &mut visited, &mut files)?;
    let mut named = BTreeMap::new();
    for path in files {
        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let name = relative
            .to_str()
            .ok_or_else(|| io::Error::other("Non-UTF-8 build identity path."))?
            .replace('\\', "/");
        println!("cargo:rerun-if-changed={}", path.display());
        named.insert(name, path);
    }
    let mut hash = CanonicalHasher::new("fpas-analysis-core-v1");
    hash.count(named.len());
    for (name, path) in named {
        hash.text(&name);
        hash.field(&fs::read(path)?);
    }
    let configuration = std::env::vars()
        .filter(|(key, _)| {
            key == "TARGET" || key.starts_with("CARGO_CFG_") || key.starts_with("CARGO_FEATURE_")
        })
        .collect::<BTreeMap<_, _>>();
    hash.count(configuration.len());
    for (key, value) in configuration {
        hash.text(&key);
        hash.text(&value);
    }
    println!("cargo:rustc-env=FPAS_ANALYSIS_BUILD_ID={}", hash.finish());
    Ok(())
}

fn collect_package(
    root: &Path,
    package: &Path,
    shared: &toml::Table,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    // Path dependencies contain `..`; remove only after filesystem resolution.
    let package = fs::canonicalize(package)?;
    let canonical_root = fs::canonicalize(root)?;
    let relative = package
        .strip_prefix(&canonical_root)
        .map_err(io::Error::other)?;
    let package = root.join(relative);
    if !visited.insert(package.clone()) {
        return Ok(());
    }
    let manifest_path = package.join("Cargo.toml");
    let value = manifest(&manifest_path)?;
    files.insert(manifest_path);
    for directory in ["src", "build"] {
        collect_sources(&package.join(directory), files)?;
    }
    let build = value
        .get("package")
        .and_then(|p| p.get("build"))
        .and_then(toml::Value::as_str)
        .unwrap_or("build.rs");
    if package.join(build).is_file() {
        files.insert(package.join(build));
    }
    let mut dependencies = Vec::new();
    local_dependencies(&value, shared, &mut dependencies);
    for (relative, workspace) in dependencies {
        let base = if workspace { root } else { &package };
        collect_package(root, &base.join(relative), shared, visited, files)?;
    }
    Ok(())
}

fn local_dependencies(value: &toml::Table, shared: &toml::Table, output: &mut Vec<(String, bool)>) {
    for key in ["dependencies", "build-dependencies"] {
        if let Some(dependencies) = value.get(key).and_then(toml::Value::as_table) {
            for (name, dependency) in dependencies {
                let workspace =
                    dependency.get("workspace").and_then(toml::Value::as_bool) == Some(true);
                let declaration = if workspace {
                    shared.get(name).unwrap_or(dependency)
                } else {
                    dependency
                };
                if let Some(path) = declaration.get("path").and_then(toml::Value::as_str) {
                    output.push((path.into(), workspace));
                }
            }
        }
    }
    if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            local_dependencies(target, shared, output);
        }
    }
}

fn collect_sources(directory: &Path, files: &mut BTreeSet<PathBuf>) -> io::Result<()> {
    if !directory.is_dir() {
        return Ok(());
    }
    println!("cargo:rerun-if-changed={}", directory.display());
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_sources(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.insert(path);
        }
    }
    Ok(())
}

fn manifest(path: &Path) -> io::Result<toml::Table> {
    toml::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
}
