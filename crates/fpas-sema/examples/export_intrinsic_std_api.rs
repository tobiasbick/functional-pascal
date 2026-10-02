//! Generates editor-only FPAS declarations for intrinsic `Std.*` units.

#[path = "export_intrinsic_std_api/documentation.rs"]
mod documentation;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[path = "export_intrinsic_std_api/qualifiers.rs"]
mod qualifiers;
#[path = "export_intrinsic_std_api/render.rs"]
mod render;

use fpas_sema::{intrinsic_std_symbols, intrinsic_std_units};
use render::render_unit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("fpas-sema must live below the repository root")?;
    let documentation = documentation_rows(&repository.join("docs/pascal/std"))?;
    if !documentation.contains_key(&(String::from("Std.Arrays"), String::from("all"))) {
        return Err("Std.Arrays.All documentation row is missing".into());
    }
    let output = repository.join("lib/api/Std");
    fs::create_dir_all(&output)?;

    for unit in intrinsic_std_units() {
        let path = output.join(format!("{}.fpas", unit.trim_start_matches("Std.")));
        let source = render_unit(unit, &intrinsic_std_symbols(unit), &documentation)
            .map_err(io::Error::other)?;
        let (parsed, diagnostics) = fpas_parser::parse_compilation_unit(&source);
        if !diagnostics.is_empty() {
            return Err(format!("generated {} is invalid: {diagnostics:?}", path.display()).into());
        }
        let source = fpas_fmt::format_source(&source, &parsed)?;
        fs::write(path, source)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct DocumentationRow {
    kind: String,
    signature: String,
    summary: String,
}

fn documentation_rows(root: &Path) -> io::Result<HashMap<(String, String), DocumentationRow>> {
    let mut paths = Vec::new();
    markdown_files(root, &mut paths)?;
    let known_units = intrinsic_std_units();
    let mut rows = HashMap::new();
    for path in paths {
        let source = fs::read_to_string(path)?;
        let heading = source.lines().find(|line| line.starts_with("# "));
        let Some(unit) = known_units
            .iter()
            .find(|unit| heading.is_some_and(|line| line.contains(**unit)))
        else {
            continue;
        };
        for line in source.lines().filter(|line| line.starts_with('|')) {
            let columns = line.split('|').map(str::trim).collect::<Vec<_>>();
            let Some(kind) = columns.get(1).copied() else {
                continue;
            };
            if !matches!(kind, "const" | "function" | "procedure" | "type") {
                continue;
            }
            let Some(signature) = columns.get(2).and_then(|value| inline_code(value)) else {
                continue;
            };
            let name = signature
                .split(['(', ':'])
                .next()
                .unwrap_or(signature)
                .trim()
                .to_ascii_lowercase();
            rows.entry(((*unit).to_owned(), name))
                .or_insert_with(|| DocumentationRow {
                    kind: kind.to_owned(),
                    signature: signature.to_owned(),
                    summary: columns.get(3).copied().unwrap_or_default().to_owned(),
                });
        }
    }
    Ok(rows)
}

fn markdown_files(directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            markdown_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            output.push(path);
        }
    }
    Ok(())
}

fn inline_code(value: &str) -> Option<&str> {
    let start = value.find('`')? + 1;
    let end = value.get(start..)?.find('`')? + start;
    value.get(start..end)
}
