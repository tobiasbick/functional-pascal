//! Hash compiler manifests and their locked production/build dependency closure.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::Path;

use toml::{Table, Value};

pub(super) fn hash(root: &Path, crates: &[&str], hasher: &mut blake3::Hasher) -> io::Result<()> {
    let mut workspace = read(&root.join("Cargo.toml"))?;
    let shared = workspace
        .get("workspace")
        .and_then(|value| value.get("dependencies"))
        .and_then(Value::as_table)
        .cloned()
        .unwrap_or_default();
    let mut shared_used = BTreeSet::new();
    let mut direct = BTreeMap::new();
    for name in crates {
        let mut manifest = read(&root.join("crates").join(name).join("Cargo.toml"))?;
        let mut names = BTreeSet::new();
        production_manifest(&mut manifest, &shared, &mut shared_used, &mut names);
        feed(hasher, name, &Value::Table(manifest).to_string());
        direct.insert(*name, names);
    }
    if let Some(workspace) = workspace.get_mut("workspace").and_then(Value::as_table_mut) {
        workspace.remove("members");
        workspace.remove("default-members");
        workspace.remove("exclude");
        if let Some(dependencies) = workspace
            .get_mut("dependencies")
            .and_then(Value::as_table_mut)
        {
            dependencies.retain(|name, _| shared_used.contains(name));
        }
    }
    feed(hasher, "workspace", &Value::Table(workspace).to_string());

    let lock = read(&root.join("Cargo.lock"))?;
    let packages = lock
        .get("package")
        .and_then(Value::as_array)
        .ok_or_else(|| io::Error::other("Cargo.lock has no packages"))?;
    let mut pending = Vec::new();
    for (name, wanted) in direct {
        let package = resolve(packages, name)?;
        pending.extend(
            edges(&packages[package])
                .filter(|edge| wanted.contains(edge.split_whitespace().next().unwrap_or_default()))
                .map(str::to_owned),
        );
    }
    let mut visited = BTreeSet::new();
    while let Some(edge) = pending.pop() {
        let index = resolve(packages, &edge)?;
        let package = &packages[index];
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if crates.contains(&name) || !visited.insert(index) {
            continue;
        }
        pending.extend(edges(package).map(str::to_owned));
    }
    for index in visited {
        feed(hasher, "dependency", &packages[index].to_string());
    }
    Ok(())
}

fn production_manifest(
    manifest: &mut Table,
    shared: &Table,
    shared_used: &mut BTreeSet<String>,
    names: &mut BTreeSet<String>,
) {
    for key in ["dev-dependencies", "test", "bench", "example"] {
        manifest.remove(key);
    }
    for kind in ["dependencies", "build-dependencies"] {
        if let Some(dependencies) = manifest.get(kind).and_then(Value::as_table) {
            for (name, value) in dependencies {
                let declaration = if value.get("workspace").and_then(Value::as_bool) == Some(true) {
                    shared_used.insert(name.clone());
                    shared.get(name).unwrap_or(value)
                } else {
                    value
                };
                names.insert(
                    declaration
                        .get("package")
                        .and_then(Value::as_str)
                        .unwrap_or(name)
                        .to_owned(),
                );
            }
        }
    }
    if let Some(targets) = manifest.get_mut("target").and_then(Value::as_table_mut) {
        for target in targets
            .iter_mut()
            .filter_map(|(_, target)| target.as_table_mut())
        {
            production_manifest(target, shared, shared_used, names);
        }
        targets.retain(|_, target| target.as_table().is_none_or(|target| !target.is_empty()));
    }
}

fn edges(package: &Value) -> impl Iterator<Item = &str> {
    package
        .get("dependencies")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn resolve(packages: &[Value], reference: &str) -> io::Result<usize> {
    let mut parts = reference.split_whitespace();
    let name = parts.next().unwrap_or_default();
    let version = parts.next();
    let source = parts.next().map(|source| source.trim_matches(['(', ')']));
    let mut candidates = packages.iter().enumerate().filter(|(_, package)| {
        package.get("name").and_then(Value::as_str) == Some(name)
            && version.is_none_or(|version| {
                package.get("version").and_then(Value::as_str) == Some(version)
            })
            && source
                .is_none_or(|source| package.get("source").and_then(Value::as_str) == Some(source))
    });
    let index = candidates
        .next()
        .map(|(index, _)| index)
        .ok_or_else(|| io::Error::other(format!("missing locked dependency {reference}")))?;
    if candidates.next().is_some() {
        return Err(io::Error::other(format!(
            "ambiguous locked dependency {reference}"
        )));
    }
    Ok(index)
}

fn read(path: &Path) -> io::Result<Table> {
    println!("cargo:rerun-if-changed={}", path.display());
    toml::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
}

fn feed(hasher: &mut blake3::Hasher, name: &str, text: &str) {
    hasher.update(&(name.len() as u64).to_le_bytes());
    hasher.update(name.as_bytes());
    hasher.update(&(text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
}
