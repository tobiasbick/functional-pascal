//! Validates the VS Code snippets against the parser and formatter.

use std::{collections::HashMap, fs, path::PathBuf};

use fpas_fmt::format_compilation_unit;
use fpas_parser::parse_compilation_unit;
use serde::Deserialize;

#[derive(Deserialize)]
struct Snippet {
    body: SnippetBody,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SnippetBody {
    Line(String),
    Lines(Vec<String>),
}

impl SnippetBody {
    fn source(&self) -> String {
        match self {
            Self::Line(line) => line.clone(),
            Self::Lines(lines) => lines.join("\n"),
        }
    }
}

#[test]
fn all_vscode_snippets_parse_format_and_reparse() {
    let path = repository_root().join("editors/vscode/snippets/fpas.json");
    let source = fs::read_to_string(&path).expect("read VS Code snippets");
    let snippets: HashMap<String, Snippet> =
        serde_json::from_str(&source).expect("parse VS Code snippets");

    assert!(snippets.len() >= 10);
    for (name, snippet) in snippets {
        let expanded = expand_defaults(&snippet.body.source());
        let compilation = wrap_snippet(&name, &expanded);
        let (unit, diagnostics) = parse_compilation_unit(&compilation);
        assert!(
            diagnostics.is_empty(),
            "snippet {name:?} does not parse:\n{compilation}\n{diagnostics:#?}"
        );

        let formatted = format_compilation_unit(&unit);
        let (_, formatted_diagnostics) = parse_compilation_unit(&formatted);
        assert!(
            formatted_diagnostics.is_empty(),
            "formatted snippet {name:?} does not parse:\n{formatted}\n{formatted_diagnostics:#?}"
        );
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("fpas-lsp crate is inside the repository")
        .to_path_buf()
}

fn wrap_snippet(name: &str, source: &str) -> String {
    match name {
        "Program" | "Unit" => source.to_owned(),
        "Function declaration" | "Procedure declaration" | "Record type" | "Mutable variable" => {
            format!("program SnippetHost;\n\n{source}\n\nbegin\nend.")
        }
        _ => format!("program SnippetHost;\n\nbegin\n{source}\nend."),
    }
}

fn expand_defaults(source: &str) -> String {
    let mut result = String::new();
    let mut rest = source;
    while let Some(index) = rest.find('$') {
        result.push_str(&rest[..index]);
        rest = &rest[index..];
        if rest.starts_with("$0") {
            rest = &rest[2..];
            continue;
        }
        if let Some(placeholder) = rest.strip_prefix("${")
            && let Some(end) = placeholder.find('}')
        {
            let contents = &placeholder[..end];
            if let Some((_, default)) = contents.split_once(':') {
                result.push_str(default);
            }
            rest = &placeholder[end + 1..];
            continue;
        }
        result.push('$');
        rest = &rest[1..];
    }
    result.push_str(rest);
    result
}
