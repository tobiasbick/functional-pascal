//! Compiler identity coverage and dependency boundaries for portable tooling.

#![allow(
    clippy::expect_used,
    reason = "workspace manifests and the build-script declaration are required test inputs"
)]

use std::collections::BTreeSet;
use std::path::Path;

const COMPILER_IDENTITY_SOURCE: &str = include_str!("../build/compiler_identity.rs");

fn dependencies(name: &str) -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates directory");
    let path = root.join(name).join("Cargo.toml");
    let text = std::fs::read_to_string(path).expect("workspace manifest");
    let manifest: toml::Table = toml::from_str(&text).expect("valid workspace manifest");
    let mut dependencies = BTreeSet::new();
    let mut collect = |table: Option<&toml::Value>| {
        if let Some(table) = table.and_then(toml::Value::as_table) {
            for (name, declaration) in table {
                let package = declaration
                    .get("package")
                    .and_then(toml::Value::as_str)
                    .unwrap_or(name);
                dependencies.insert(package.to_owned());
            }
        }
    };
    collect(manifest.get("dependencies"));
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values() {
            collect(target.get("dependencies"));
        }
    }
    dependencies
}

fn normal_workspace_graph(root: &str) -> BTreeSet<String> {
    let mut visited = BTreeSet::new();
    let mut pending = vec![root.to_owned()];
    while let Some(name) = pending.pop() {
        if visited.insert(name.clone()) {
            pending.extend(
                dependencies(&name)
                    .into_iter()
                    .filter(|dependency| dependency.starts_with("fpas-")),
            );
        }
    }
    visited
}

#[test]
fn compiler_identity_covers_every_normal_workspace_dependency() {
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
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    assert_eq!(actual, normal_workspace_graph("fpas-build"));
}

#[test]
fn compiler_and_analysis_do_not_depend_on_execution_or_host_transports() {
    let forbidden = [
        "fpas-runtime",
        "fpas-acp",
        "fpas-acp-codec",
        "fpas-mcp",
        "agent-client-protocol",
        "rmcp",
        "tokio",
        "crossterm",
        "rustls",
        "socket2",
        "process-wrap",
    ];
    for root in [
        "fpas-build",
        "fpas-compiler",
        "fpas-sema",
        "fpas-code",
        "fpas-builtins",
        "fpas-program",
        "fpas-linker",
        "fpas-unit",
        "fpas-project",
        "fpas-language-service",
    ] {
        for name in normal_workspace_graph(root) {
            for dependency in dependencies(&name) {
                assert!(
                    !forbidden.contains(&dependency.as_str()),
                    "{root} reaches {dependency} through the normal dependencies of {name}"
                );
            }
        }
    }
}

#[test]
fn portable_code_has_only_data_dependencies() {
    assert_eq!(
        dependencies("fpas-code"),
        ["fpas-diagnostics", "num_enum", "serde"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
}
