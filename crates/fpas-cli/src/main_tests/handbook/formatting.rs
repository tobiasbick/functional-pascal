//! Canonical examples and structural consumer coverage from actual Markdown.

use super::{document, fences};
use fpas_fmt::format_source;
use fpas_parser::parse_compilation_unit;
use std::{fs, path::Path};

#[test]
fn all_complete_handbook_sources_parse_format_and_reparse() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/pascal");
    let mut count = 0;
    visit_markdown(&root, &mut |path, text| {
        for source in fences::pascal_sources(text) {
            if is_complete(source) {
                assert_formatting(&path.display().to_string(), source, false);
                count += 1;
            }
        }
    });
    assert!(
        count >= 58,
        "handbook sources unexpectedly disappeared: {count}"
    );
}

#[test]
fn formatter_style_and_authoring_skeletons_are_canonical_output() {
    for path in [
        "docs/pascal/tools/fmt-style.md",
        ".agents/skills/fpas-authoring/SKILL.md",
    ] {
        let text = document(path);
        let sources = fences::pascal_sources(&text).collect::<Vec<_>>();
        assert_eq!(sources.len(), 3, "{path}: expected three skeletons");
        for source in sources {
            assert_formatting(path, source, true);
        }
    }
}

#[test]
fn documented_empty_units_and_callback_boundaries_survive_crlf_and_unicode_comments() {
    let callbacks = super::example("docs/pascal/language/functions/closures.md", 1);
    for source in ["unit Empty; end unit;".to_owned(), callbacks] {
        let source = format!("// Grüße 東京: end. uses Null shl\n{source}// tail");
        let source = source.replace('\n', "\r\n");
        assert_formatting("CRLF and Unicode", &source, false);
    }
}

fn is_complete(source: &str) -> bool {
    source.split_whitespace().next().is_some_and(|word| {
        word.eq_ignore_ascii_case("program") || word.eq_ignore_ascii_case("unit")
    })
}

fn assert_formatting(label: &str, source: &str, canonical: bool) {
    let (unit, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{label}: {diagnostics:?}\n{source}");
    let formatted = format_source(source, &unit).expect("matching source and AST");
    if canonical {
        assert_eq!(formatted, source.replace("\r\n", "\n"), "{label}");
    }
    let (again, diagnostics) = parse_compilation_unit(&formatted);
    assert!(diagnostics.is_empty(), "{label}: {diagnostics:?}");
    assert_eq!(
        format_source(&formatted, &again).expect("formatted source and AST"),
        formatted,
        "{label}: second formatting must be identical"
    );
    assert_eq!(
        comments(source),
        comments(&formatted),
        "{label}: every comment must survive"
    );
}

fn comments(source: &str) -> Vec<String> {
    fpas_lexer::collect_comments(source)
        .iter()
        .map(|comment| {
            comment
                .text(source)
                .expect("lexer span")
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn visit_markdown(path: &Path, visit: &mut dyn FnMut(&Path, &str)) {
    let mut paths = fs::read_dir(path)
        .expect("documentation directory")
        .map(|entry| entry.expect("documentation entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            visit_markdown(&path, visit);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            visit(&path, &fs::read_to_string(&path).expect("Markdown source"));
        }
    }
}
