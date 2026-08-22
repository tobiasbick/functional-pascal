//! Integration tests for resolving navigation targets.

#![allow(
    clippy::expect_used,
    reason = "navigation fixtures use explicit source offsets"
)]

mod support;

use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn repeated_member_names_resolve_each_chain_component() {
    let temp = TempDirectory::new("navigation-repeated-members");
    let source = r#"program Repeated;

type Leaf = record
  public Value: integer;
end;

type Branch = record
  public Child: Leaf;
end;

type RootType = record
  public Child: Branch;
end;

var Root: RootType;

begin
  var Result: integer := Root.Child.Child.Value
end.
"#;
    let path = temp.write("repeated.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let chain = source.find("Root.Child.Child.Value").expect("member chain");
    let first_child = chain + "Root.".len();
    let second_child = chain + "Root.Child.".len();

    let first = service
        .definitions(&path, first_child)
        .expect("first child definition")
        .value;
    let second = service
        .definitions(&path, second_child)
        .expect("second child definition")
        .value;
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(second.len(), 1, "{second:?}");
    assert_ne!(
        first[0].symbol.selection_span,
        second[0].symbol.selection_span
    );
    assert!(first[0].symbol.detail.ends_with(": Branch"), "{first:?}");
    assert!(second[0].symbol.detail.ends_with(": Leaf"), "{second:?}");

    let hover = service
        .hover(&path, second_child)
        .expect("second child hover")
        .value
        .expect("resolved hover");
    assert!(hover.contents.ends_with(": Leaf"), "{hover:?}");

    let completions = service
        .completions(&path, chain + "Root.Child.Child.".len())
        .expect("leaf completion")
        .value;
    assert!(completions.iter().any(|item| item.label == "Value"));
}

#[test]
fn hierarchical_unit_resolution_is_independent_of_source_and_uses_order() {
    for (label, a_path, ab_path, uses) in [
        ("forward", "src/01-a.fpas", "src/02-ab.fpas", "A, A.B"),
        ("reverse", "src/02-a.fpas", "src/01-ab.fpas", "A.B, A"),
    ] {
        let temp = TempDirectory::new(label);
        let manifest = project_manifest(&temp);
        temp.write(
            a_path,
            "unit A;\n\npublic function Other(): integer;\nbegin return 1 end;\n",
        );
        let ab = temp.write(
            ab_path,
            "unit A.B;\n\npublic function Target(): integer;\nbegin return 2 end;\n",
        );
        let main_source = format!(
            "program App;\n\nuses {uses};\n\nbegin\n  var Value: integer := A.B.Target()\nend.\n"
        );
        let main = temp.write("src/main.fpas", &main_source);
        let mut service = LanguageService::load(&manifest);

        let definitions = service
            .definitions(&main, main_source.find("Target").expect("qualified target"))
            .expect("hierarchical unit definition")
            .value;
        assert_eq!(definitions.len(), 1, "{label}: {definitions:?}");
        assert_eq!(definitions[0].path, ab);
    }
}

#[test]
fn genuinely_ambiguous_qualified_candidates_do_not_pick_source_order() {
    let temp = TempDirectory::new("navigation-qualified-ambiguity");
    let manifest = project_manifest(&temp);
    temp.write(
        "src/a.fpas",
        "unit A;\n\npublic type B = record\n  public C: integer;\nend;\n",
    );
    temp.write("src/ab.fpas", "unit A.B;\n\npublic var C: integer := 1;\n");
    let main_source =
        "program App;\n\nuses A, A.B;\n\nbegin\n  var Value: integer := A.B.C\nend.\n";
    let main = temp.write("src/main.fpas", main_source);
    let mut service = LanguageService::load(&manifest);

    let definitions = service
        .definitions(&main, main_source.rfind('C').expect("ambiguous candidate"))
        .expect("ambiguous definition query")
        .value;
    assert!(definitions.is_empty(), "{definitions:?}");
}

fn project_manifest(temp: &TempDirectory) -> std::path::PathBuf {
    temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    )
}
