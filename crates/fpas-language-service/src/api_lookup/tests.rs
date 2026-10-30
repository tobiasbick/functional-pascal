//! Catalog lookup and handbook extraction without loaded sources.

use super::declarations::handbook_path;
use super::native;
use super::{ApiEntryKind, ApiLookup};
use crate::{LanguageService, WorkspaceContext};

fn lookup(query: &str) -> ApiLookup {
    let root = std::env::temp_dir();
    LanguageService::new(WorkspaceContext::loose(&root))
        .api_catalog()
        .expect("catalog without sources")
        .lookup(query)
}

#[test]
fn owner_qualified_operation_is_case_insensitive() {
    let result = lookup("STRING.trimleft");
    let [entry] = result.matches.as_slice() else {
        panic!("{result:?}");
    };
    assert_eq!(entry.name, "string.TrimLeft");
    assert_eq!(entry.kind, ApiEntryKind::Operation);
    assert_eq!(entry.signature, "string.TrimLeft(): string");
    assert_eq!(
        entry.handbook.as_deref(),
        Some("docs/pascal/language/types/string/README.md")
    );
    assert!(entry.location.is_none());
}

#[test]
fn bare_operation_name_lists_every_receiver() {
    let names = lookup("Length")
        .matches
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "Std.Bytes.ByteBuffer.Length",
            "string.Length",
            "array.Length",
            "dict.Length"
        ]
    );
}

#[test]
fn factories_are_marked_and_string_array_join_is_found_on_array() {
    let factory = lookup("string.Chr");
    assert_eq!(factory.matches[0].kind, ApiEntryKind::Factory);
    let join = lookup("array.Join");
    assert_eq!(join.matches.len(), 1);
    assert!(
        join.matches[0]
            .documentation
            .as_deref()
            .is_some_and(|text| text.contains("`array of string` only"))
    );
}

#[test]
fn every_catalog_operation_resolves_by_its_listed_name() {
    for name in native::names() {
        assert!(
            lookup(&name).matches.iter().any(|entry| entry.name == name),
            "{name}"
        );
    }
}

#[test]
fn unknown_name_suggests_close_operations() {
    let result = lookup("string.TrimLfet");
    assert!(result.matches.is_empty());
    assert_eq!(result.suggestions[0], "string.TrimLeft");
}

#[test]
fn handbook_path_is_taken_from_the_unit_comment() {
    assert_eq!(
        handbook_path("File access.\n\nHandbook: `docs/pascal/std/host/fs.md`.").as_deref(),
        Some("docs/pascal/std/host/fs.md")
    );
    assert_eq!(handbook_path("No page."), None);
}
