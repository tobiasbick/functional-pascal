//! Integration tests for command-line API lookup over the repository standard library and projects.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixtures use expect to keep lookup assertions focused"
)]

mod support;

use std::path::{Path, PathBuf};

use fpas_language_service::{ApiCatalog, ApiEntry, ApiEntryKind, LanguageService, SymbolKind};
use support::TempDirectory;

fn standard_library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib")
}

fn service_for(root: &Path) -> LanguageService {
    LanguageService::load_with_standard_library(root, &standard_library())
        .expect("repository standard library")
}

fn catalog_for(root: &Path) -> ApiCatalog {
    service_for(root).api_catalog().expect("catalog")
}

fn single(catalog: &ApiCatalog, query: &str) -> ApiEntry {
    let result = catalog.lookup(query);
    let [entry] = result.matches.as_slice() else {
        panic!("expected one match for {query}: {result:?}");
    };
    entry.clone()
}

const GEOMETRY: &str = "// Plane geometry for the demo.
//
// Handbook: `docs/pascal/language/types/records.md`.
unit Demo.Geometry;

// A point in the plane.
public type TPoint = record
  public X: real := 0.0;
  public Y: real := 0.0;

  // Manhattan distance to `Other`.
  function Distance(Self: TPoint; Other: TPoint): real;
  begin
    return Abs(Self.X - Other.X) + Abs(Self.Y - Other.Y);
  end function;
end record;

// Internal helper.
function Twice(Value: integer): integer;
begin
  return Value * 2;
end function;

// The answer.
public function Answer(): integer;
begin
  return Twice(21);
end function;
end unit;
";

fn geometry_project(temp: &TempDirectory) -> (PathBuf, PathBuf) {
    temp.write(
        "demo.fpasprj",
        "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n",
    );
    let main = temp.write(
        "src/main.fpas",
        "program App;\n\nuses Demo.Geometry as Geometry;\n\nbegin\n  const Value: integer := Geometry.Answer();\nend.\n",
    );
    let unit = temp.write("src/geometry.fpas", GEOMETRY);
    (main, unit)
}

#[test]
fn intrinsic_routine_matches_hover_documentation_and_unit_handbook() {
    let temp = TempDirectory::new("api-lookup-intrinsic");
    let catalog = catalog_for(temp.path());

    let entry = single(&catalog, "Std.Fs.ReadText");
    assert_eq!(entry.kind, ApiEntryKind::Declaration(SymbolKind::Function));
    assert_eq!(
        entry.documentation.as_deref(),
        Some(
            "Reads UTF-8 text.\n\nParameters:\n- `Path`: File or directory path processed by the operation."
        )
    );
    assert_eq!(
        entry.handbook.as_deref(),
        Some("docs/pascal/std/host/fs.md")
    );
    let location = entry.location.expect("generated declaration location");
    assert!(
        location.path.ends_with("lib/api/Std/Fs.fpas"),
        "{location:?}"
    );

    assert_eq!(single(&catalog, "fs.readtext").name, "Std.Fs.ReadText");
    let names = catalog
        .lookup("READTEXT")
        .matches
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["Std.Console.ReadText", "Std.Fs.ReadText"]);
}

#[test]
fn unit_lists_public_declarations_with_their_first_comment_line() {
    let temp = TempDirectory::new("api-lookup-unit");
    let catalog = catalog_for(temp.path());

    let unit = single(&catalog, "Std.Fs");
    assert_eq!(unit.kind, ApiEntryKind::Declaration(SymbolKind::Unit));
    assert!(
        unit.documentation
            .as_deref()
            .is_some_and(|text| text.starts_with("Basic blocking filesystem operations")),
        "{unit:?}"
    );
    let read = unit
        .members
        .iter()
        .find(|member| member.name == "ReadText")
        .expect("ReadText member");
    assert_eq!(read.summary.as_deref(), Some("Reads UTF-8 text."));
}

#[test]
fn source_standard_library_units_are_found_without_a_project_and_hide_private_declarations() {
    let temp = TempDirectory::new("api-lookup-source-std");
    let catalog = catalog_for(temp.path());

    let http = single(&catalog, "Std.Http");
    assert_eq!(
        http.handbook.as_deref(),
        Some("docs/pascal/std/network/http.md")
    );
    assert!(!http.members.is_empty());
    let uri = single(&catalog, "Std.Net.Uri");
    let members = uri
        .members
        .iter()
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(members, ["Uri", "Parse"]);
    assert!(catalog.lookup("Std.Net.Uri.ParsePort").matches.is_empty());
}

#[test]
fn every_intrinsic_symbol_resolves_by_its_qualified_name() {
    let temp = TempDirectory::new("api-lookup-all-intrinsic");
    let catalog = catalog_for(temp.path());
    for unit in fpas_sema::intrinsic_std_units() {
        assert!(!catalog.lookup(unit).matches.is_empty(), "{unit}");
        for symbol in fpas_sema::intrinsic_std_symbols(unit) {
            let result = catalog.lookup(&symbol.qualified_name);
            assert!(
                result
                    .matches
                    .iter()
                    .any(|entry| entry.name.eq_ignore_ascii_case(&symbol.qualified_name)),
                "{} is not found",
                symbol.qualified_name
            );
        }
    }
}

#[test]
fn project_declarations_match_hover_including_record_methods_and_private_routines() {
    let temp = TempDirectory::new("api-lookup-project");
    let (_main, unit) = geometry_project(&temp);
    let mut service = service_for(temp.path());
    let catalog = service.api_catalog().expect("catalog");

    let method = single(&catalog, "TPoint.Distance");
    assert_eq!(method.name, "Demo.Geometry.TPoint.Distance");
    assert_eq!(method.kind, ApiEntryKind::Declaration(SymbolKind::Method));
    assert_eq!(
        method.documentation.as_deref(),
        Some("Manhattan distance to `Other`.")
    );
    assert_eq!(
        method.handbook.as_deref(),
        Some("docs/pascal/language/types/records.md")
    );
    let location = method.location.expect("method location");
    assert_eq!((location.line, location.column), (12, 12));

    let offset = GEOMETRY.find("Twice(21)").expect("Twice use");
    let hover = service
        .hover(&unit, offset)
        .expect("hover")
        .value
        .expect("Twice hover");
    let helper = single(&catalog, "Geometry.Twice");
    assert_eq!(helper.documentation, hover.documentation);
    assert_eq!(helper.signature, hover.contents);
}

#[test]
fn a_project_with_type_errors_is_still_searchable() {
    let temp = TempDirectory::new("api-lookup-broken");
    geometry_project(&temp);
    temp.write(
        "src/broken.fpas",
        "unit Demo.Broken;\n\n// Returns text where a number is declared.\npublic function Wrong(): integer;\nbegin\n  return 'text';\nend function;\nend unit;\n",
    );
    let catalog = catalog_for(temp.path());
    let entry = single(&catalog, "Demo.Broken.Wrong");
    assert_eq!(
        entry.documentation.as_deref(),
        Some("Returns text where a number is declared.")
    );
}

#[test]
fn ambiguous_short_names_list_every_declaration() {
    let temp = TempDirectory::new("api-lookup-ambiguous");
    geometry_project(&temp);
    temp.write(
        "src/other.fpas",
        "unit Demo.Other;\n\npublic function Answer(): integer;\nbegin\n  return 7;\nend function;\nend unit;\n",
    );
    let catalog = catalog_for(temp.path());
    let names = catalog
        .lookup("Answer")
        .matches
        .into_iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>();
    assert_eq!(names, ["Demo.Geometry.Answer", "Demo.Other.Answer"]);
}

#[test]
fn misspelled_names_suggest_declarations_and_operations() {
    let temp = TempDirectory::new("api-lookup-suggestions");
    let catalog = catalog_for(temp.path());
    let result = catalog.lookup("Fs.ReadTxt");
    assert!(result.matches.is_empty());
    assert_eq!(
        result.suggestions.first().map(String::as_str),
        Some("Std.Fs.ReadText")
    );
}
