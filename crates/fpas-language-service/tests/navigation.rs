#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "navigation fixtures use explicit assertions and source offsets"
)]

mod support;

use fpas_language_service::{DocumentSymbol, LanguageService, SymbolKind, WorkspaceContext};
use support::{TempDirectory, write_program_project};

#[test]
fn document_symbols_cover_roots_types_routines_parameters_members_and_variables() {
    let temp = TempDirectory::new("navigation-symbols");
    let source = r#"program Symbols;

type Point = record
  public X: integer;
  property LabelText: string read GetLabel;
end;

function Add(Value: integer): integer;
begin
  var Local: integer := Value;
  return Local
end;

begin
  mutable var Current: integer := 1
end.
"#;
    let path = temp.write("symbols.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let result = service.document_symbols(&path).expect("document symbols");
    let [root] = result.value.as_slice() else {
        panic!("expected one compilation-unit root")
    };
    assert_eq!(root.kind, SymbolKind::Program);
    let point = child(root, "Point");
    assert_eq!(point.kind, SymbolKind::Type);
    assert_eq!(child(point, "X").kind, SymbolKind::Field);
    assert_eq!(child(point, "LabelText").kind, SymbolKind::Property);
    let add = child(root, "Add");
    assert_eq!(add.kind, SymbolKind::Function);
    assert_eq!(child(add, "Value").kind, SymbolKind::Parameter);
    assert_eq!(child(add, "Local").kind, SymbolKind::Variable);
    assert_eq!(child(root, "Current").kind, SymbolKind::MutableVariable);
    for symbol in all_symbols(root) {
        assert_eq!(
            &source[symbol.selection_span.offset
                ..symbol.selection_span.offset + symbol.selection_span.length],
            symbol.name
        );
        assert!(symbol.full_span.offset <= symbol.selection_span.offset);
        assert!(
            symbol.selection_span.offset + symbol.selection_span.length
                <= symbol.full_span.offset + symbol.full_span.length
        );
    }
}

#[test]
fn hover_and_definition_follow_lexical_shadowing_and_ignore_non_identifiers() {
    let temp = TempDirectory::new("navigation-local");
    let source = r#"program Local;

var Value: integer := 1;

function Read(Value: integer): integer;
begin
  var Other: integer := Value;
  return Other
end;

begin
  // Value in a comment
  var Text: string := 'Value';
  var Output: integer := Read(Value)
end.
"#;
    let path = temp.write("local.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let parameter_use = source.find("Value;\n  return").expect("parameter use");
    let definition = service
        .definitions(&path, parameter_use)
        .expect("parameter definition");
    assert_eq!(definition.value.len(), 1);
    assert_eq!(definition.value[0].symbol.kind, SymbolKind::Parameter);
    let hover = service
        .hover(&path, source.rfind("Read(Value").expect("function call"))
        .expect("hover query")
        .value
        .expect("function hover");
    assert!(hover.contents.starts_with("function Read("), "{hover:?}");

    for offset in [
        source.find("Value in a comment").expect("comment"),
        source.find("'Value'").expect("string") + 1,
        source.find("Output").expect("declaration") + "Output".len(),
    ] {
        assert!(
            service
                .definitions(&path, offset)
                .expect("negative definition query")
                .value
                .is_empty()
        );
    }
}

#[test]
fn project_navigation_respects_imports_visibility_members_and_unsaved_changes() {
    let temp = TempDirectory::new("navigation-project");
    let (manifest, main, unit) = write_program_project(&temp);
    let unit_source = r#"unit Demo.Math;

public type Point = record
  public X: integer;
  Secret: integer;
end;

public function Answer(): integer;
begin
  return 42
end;

function Hidden(): integer;
begin
  return 0
end;
"#;
    std::fs::write(&unit, unit_source).expect("replace unit fixture");
    let main_source = r#"program App;

uses Demo.Math;

begin
  var A: integer := Answer();
  var B: integer := Demo.Math.Answer();
  var P: Point := record X := 0; end;
  var C: integer := P.X;
  var D: integer := Hidden()
end.
"#;
    std::fs::write(&main, main_source).expect("replace main fixture");
    let mut service = LanguageService::load(&manifest);
    assert!(
        service.workspace().project_for_source(&unit).is_some(),
        "unit={unit:?}, projects={:?}",
        service.workspace().projects()
    );
    let unit_symbols = service.document_symbols(&unit).expect("unit symbols").value;
    assert!(
        all_symbols(&unit_symbols[0])
            .iter()
            .any(|symbol| symbol.name == "Answer"),
        "{unit_symbols:?}"
    );

    for reference in ["Answer();", "Demo.Math.Answer();"] {
        let offset = main_source.find(reference).expect("public reference")
            + if reference.starts_with("Demo.") {
                "Demo.Math.".len()
            } else {
                0
            };
        let definition = service
            .definitions(&main, offset)
            .expect("cross-unit definition");
        assert_eq!(
            definition.value.len(),
            1,
            "{reference}: {definition:?}; unit={unit_symbols:?}"
        );
        assert_eq!(definition.value[0].path, unit);
        assert_eq!(definition.value[0].symbol.name, "Answer");
    }
    let unit_definition = service
        .definitions(
            &main,
            main_source.find("Math.Answer").expect("qualified unit") + 1,
        )
        .expect("qualified unit definition");
    assert_eq!(unit_definition.value[0].symbol.kind, SymbolKind::Unit);
    let completions = service
        .completions(
            &main,
            main_source.find("P.X").expect("member access") + "P.".len(),
        )
        .expect("member completion")
        .value;
    assert!(completions.iter().any(|entry| entry.label == "X"));
    assert!(!completions.iter().any(|entry| entry.label == "Secret"));
    let normal = service
        .completions(&main, main_source.find("begin").expect("body"))
        .expect("project completion")
        .value;
    assert!(normal.iter().any(|entry| entry.label == "Answer"));
    assert!(!normal.iter().any(|entry| entry.label == "Hidden"));
    assert!(
        service
            .definitions(
                &main,
                main_source.find("Hidden()").expect("private reference")
            )
            .expect("private definition query")
            .value
            .is_empty()
    );

    service
        .documents_mut()
        .open_document(&unit, 1, unit_source.replace("Answer", "Updated"))
        .expect("unsaved unit");
    service
        .documents_mut()
        .open_document(&main, 1, main_source.replace("Answer", "Updated"))
        .expect("unsaved program");
    let updated_source = service
        .documents()
        .open_snapshot(&main)
        .expect("open main")
        .source()
        .to_owned();
    let updated = service
        .definitions(
            &main,
            updated_source.find("Updated();").expect("updated use"),
        )
        .expect("unsaved definition");
    assert_eq!(updated.value[0].symbol.name, "Updated");
}

#[test]
fn unknown_private_and_outside_project_queries_are_empty() {
    let temp = TempDirectory::new("navigation-negative");
    let (manifest, main, _unit) = write_program_project(&temp);
    let outside = temp.write("outside.fpas", "program Outside; begin end.");
    let mut service = LanguageService::load(&manifest);

    let main_source = std::fs::read_to_string(&main).expect("main source");
    assert!(
        service
            .definitions(&main, main_source.find("begin").expect("keyword"))
            .expect("keyword query")
            .value
            .is_empty()
    );
    assert!(
        service
            .document_symbols(&outside)
            .expect("outside symbols")
            .value
            .is_empty()
    );
    assert!(
        service
            .completions(&outside, 0)
            .expect("outside completion")
            .value
            .is_empty()
    );
}

#[test]
fn completion_preserves_equal_import_candidates_from_distinct_units() {
    let temp = TempDirectory::new("navigation-candidates");
    let manifest = temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = temp.write(
        "src/main.fpas",
        "program App;\n\nuses Demo.First, Demo.Second;\n\nbegin\nend.\n",
    );
    temp.write(
        "src/first.fpas",
        "unit Demo.First;\n\npublic function Create(): integer;\nbegin\n  return 1\nend;\n",
    );
    temp.write(
        "src/second.fpas",
        "unit Demo.Second;\n\npublic function Create(): integer;\nbegin\n  return 2\nend;\n",
    );
    let mut service = LanguageService::load(&manifest);
    let source = std::fs::read_to_string(&main).expect("main source");

    let candidates = service
        .completions(&main, source.find("end.").expect("completion position"))
        .expect("completion candidates")
        .value
        .into_iter()
        .filter(|candidate| candidate.label == "Create")
        .collect::<Vec<_>>();

    assert_eq!(candidates.len(), 2, "{candidates:?}");
    assert_ne!(candidates[0].qualified_name, candidates[1].qualified_name);
}

#[test]
fn dependency_units_outside_library_exports_are_not_navigable() {
    let temp = TempDirectory::new("navigation-exports");
    temp.write(
        "lib/lib.fpasprj",
        r#"[project]
name = "lib"
kind = "library"

[exports]
units = ["Demo.Exported"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write(
        "lib/src/public.fpas",
        "unit Demo.Exported;\n\npublic function Visible(): integer;\nbegin return 1 end;\n",
    );
    temp.write(
        "lib/src/internal.fpas",
        "unit Demo.Internal;\n\npublic function Hidden(): integer;\nbegin return 2 end;\n",
    );
    let manifest = temp.write(
        "app/app.fpasprj",
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../lib/lib.fpasprj"]

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    let main = temp.write(
        "app/src/main.fpas",
        "program App;\n\nuses Demo.Internal;\n\nbegin\n  var Value: integer := Hidden()\nend.\n",
    );
    let mut service = LanguageService::load(&manifest);
    let source = std::fs::read_to_string(&main).expect("main source");

    assert!(
        service
            .definitions(&main, source.find("Hidden").expect("hidden use"))
            .expect("export-aware definition")
            .value
            .is_empty()
    );
}

fn child<'a>(symbol: &'a DocumentSymbol, name: &str) -> &'a DocumentSymbol {
    symbol
        .children
        .iter()
        .find(|child| child.name == name)
        .unwrap_or_else(|| panic!("missing child {name}: {:?}", symbol.children))
}

fn all_symbols(symbol: &DocumentSymbol) -> Vec<&DocumentSymbol> {
    let mut symbols = vec![symbol];
    for child in &symbol.children {
        symbols.extend(all_symbols(child));
    }
    symbols
}
