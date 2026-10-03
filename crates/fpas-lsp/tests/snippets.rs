//! Validates the VS Code snippets against the parser and formatter.

#![allow(
    clippy::expect_used,
    reason = "snippet fixtures fail immediately when repository assets are missing or malformed"
)]

use std::{collections::HashMap, fs, path::PathBuf};

use fpas_fmt::format_source;
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
    let snippets = snippets();

    assert!(snippets.len() >= 10);
    for (name, snippet) in snippets {
        let expanded = expand_defaults(&snippet.body.source());
        let compilation = wrap_snippet(&name, &expanded);
        assert_canonical_round_trip(&name, &compilation);
    }
}

fn snippets() -> HashMap<String, Snippet> {
    let path = repository_root().join("editors/vscode/snippets/fpas.json");
    let source = fs::read_to_string(&path).expect("read VS Code snippets");
    serde_json::from_str(&source).expect("parse VS Code snippets")
}

#[test]
fn obsolete_snippet_closers_and_alias_free_imports_are_rejected() {
    let snippets = snippets();
    for (name, current, obsolete, hint) in [
        ("Program", "end program;", "end.", "end program;"),
        ("Unit", "end unit;", "", "end unit"),
        (
            "Function declaration",
            "end function;",
            "end;",
            "end function",
        ),
        (
            "Procedure declaration",
            "end procedure;",
            "end;",
            "end procedure",
        ),
        ("If statement", "end if;", "end;", "end if"),
        ("For loop", "end for;", "end;", "end for"),
        ("While loop", "end while;", "end;", "end while"),
        ("Case statement", "end case;", "end;", "end case"),
        ("Import unit", " as Console", "", "as"),
    ] {
        let expanded = expand_defaults(&snippets[name].body.source());
        assert!(
            expanded.contains(current),
            "{name}: obsolete fixture has no replacement"
        );
        let compilation = wrap_snippet(name, &expanded.replace(current, obsolete));
        let (_, diagnostics) = parse_compilation_unit(&compilation);
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|text| text.contains(hint))),
            "{name}: missing {hint:?} migration hint: {diagnostics:#?}"
        );
    }
}

#[test]
fn snippet_defaults_accept_case_variants_crlf_and_comments_with_retired_words() {
    for (name, snippet) in snippets() {
        let expanded = expand_defaults(&snippet.body.source()).to_uppercase();
        let source = wrap_snippet(&name, &expanded);
        let source =
            format!("// Grüße 東京: uses Null shl end.\n{source} // tail").replace('\n', "\r\n");
        let formatted = assert_canonical_round_trip(&name, &source);
        assert!(formatted.contains("// Grüße 東京: uses Null shl end."));
        assert!(formatted.contains("// tail"));
    }
}

fn assert_canonical_round_trip(name: &str, source: &str) -> String {
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{name}: {diagnostics:#?}\n{source}");
    let formatted = format_source(source, &unit).expect("snippet source and AST");
    let (again, diagnostics) = parse_compilation_unit(&formatted);
    assert!(
        diagnostics.is_empty(),
        "{name}: {diagnostics:#?}\n{formatted}"
    );
    assert_eq!(
        format_source(&formatted, &again).expect("formatted snippet"),
        formatted,
        "{name}"
    );
    formatted
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
        "Function declaration"
        | "Procedure declaration"
        | "Record type"
        | "Mutable variable"
        | "Import unit" => {
            format!("program SnippetHost;\n\n{source}\n\nbegin\n  null;\nend program;")
        }
        _ => format!("program SnippetHost;\n\nbegin\n{source}\nend program;"),
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
