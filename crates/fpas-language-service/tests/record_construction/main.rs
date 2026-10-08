//! Constructor fields retain canonical identities through record and import aliases.

#![allow(
    clippy::expect_used,
    reason = "editor fixtures assert exact source positions"
)]

#[path = "../support/mod.rs"]
mod support;

use fpas_language_service::LanguageService;
use support::TempDirectory;

const UNIT: &str = "unit Demo.Model;
  public type Point = record public X: integer; public Y: integer := 2; end record;
  public type Location = Point;
  public type Hidden = record public X: integer; Secret: integer := 0; end record;
  public function Origin(): Point; begin return Point(X := 0); end function;
end unit;";

struct Fixture {
    _temp: TempDirectory,
    service: LanguageService,
    main: std::path::PathBuf,
    unit: std::path::PathBuf,
}

fn fixture(source: &str) -> Fixture {
    let temp = TempDirectory::new("record-construction");
    let manifest = temp.write("demo.fpasprj", "[project]\nname = \"demo\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n");
    let unit = temp.write("src/model.fpas", UNIT);
    let main = temp.write("src/main.fpas", source);
    Fixture {
        service: LanguageService::load(&manifest),
        _temp: temp,
        main,
        unit,
    }
}

#[test]
fn field_labels_resolve_and_hover_through_imported_and_local_type_aliases() {
    let source = "program App; uses Demo.Model as M; type Position = M.Location; begin const P: Position := Position(y := 4, X := 1); end.";
    let mut f = fixture(source);
    assert!(
        f.service
            .analyze_document(&f.main)
            .expect("analysis")
            .diagnostics()
            .is_empty()
    );
    for (label, field) in [("y :=", "Y"), ("X :=", "X")] {
        let offset = source.find(label).expect("label");
        let definitions = f
            .service
            .definitions(&f.main, offset)
            .expect("field definition")
            .value;
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].path, f.unit);
        assert_eq!(
            definitions[0].symbol.qualified_name,
            format!("Demo.Model.Point.{field}")
        );
        let hover = f
            .service
            .hover(&f.main, offset)
            .expect("field hover")
            .value
            .expect("hover");
        assert!(
            hover.contents.contains(&format!("field {field}: integer")),
            "{hover:?}"
        );
    }
}

#[test]
fn record_field_rename_updates_constructor_labels_in_units_and_alias_calls() {
    let source = "program App; uses Demo.Model as M; type Position = M.Location; begin const P: M.Point := M.Point(X := 1); const Q: Position := Position(x := 3); end.";
    let mut f = fixture(source);
    let edits = f
        .service
        .rename(&f.main, source.find("X :=").expect("label"), "Horizontal")
        .expect("field rename")
        .value;
    assert_eq!(edits.len(), 4, "{edits:#?}");
    assert_eq!(edits.iter().filter(|edit| edit.path == f.main).count(), 2);
    assert_eq!(edits.iter().filter(|edit| edit.path == f.unit).count(), 2);
}

#[test]
fn constructor_signature_reports_declared_defaults_and_named_active_field() {
    let source = "program App; uses Demo.Model as M; begin const P: M.Location := M.Location(y := 4, X := 1); end.";
    let mut f = fixture(source);
    for (needle, active) in [("4,", 1), ("1);", 0)] {
        let help = f
            .service
            .signature_help(&f.main, source.find(needle).expect("value") + 1)
            .expect("signature help")
            .value
            .expect("record signature");
        assert_eq!(help.active_parameter, Some(active));
        assert_eq!(
            help.signature.label,
            "Location(X: integer; Y: integer := 2): Location"
        );
    }
}

#[test]
fn constructor_completion_offers_unfilled_fields_and_inserts_named_syntax() {
    let source =
        "program App; uses Demo.Model as M; begin const P: M.Location := M.Location(X := 1); end.";
    let mut f = fixture(source);
    let edited = source.replace("X := 1", "X := 1, ");
    f.service
        .documents_mut()
        .open_document(&f.main, 1, edited.clone())
        .expect("open incomplete arguments");
    let candidates = f
        .service
        .completions(&f.main, edited.find("1, ").expect("argument start") + 3)
        .expect("field completion")
        .value;
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].label, "Y");
    assert_eq!(candidates[0].insert_text, "Y := ");
    assert_eq!(candidates[0].qualified_name, "Demo.Model.Point.Y");
}

#[test]
fn completion_on_existing_field_label_preserves_assignment_and_excludes_other_labels() {
    let source = "program App; uses Demo.Model as M; begin const P: M.Point := M.Point(Y := 2, X := 1); end.";
    let mut f = fixture(source);
    let candidates = f
        .service
        .completions(&f.main, source.find("X :=").expect("label") + 1)
        .expect("field completion")
        .value;
    assert_eq!(candidates.len(), 1, "{candidates:#?}");
    assert_eq!(candidates[0].label, "X");
    assert_eq!(candidates[0].insert_text, "X");
}

#[test]
fn private_record_construction_does_not_expose_labels_or_signature_outside_its_unit() {
    let source =
        "program App; uses Demo.Model as M; begin const P: M.Hidden := M.Hidden(X := 1); end.";
    let mut f = fixture(source);
    assert!(
        f.service
            .definitions(&f.main, source.find("X :=").expect("field"))
            .expect("private constructor")
            .value
            .is_empty()
    );
    assert!(
        f.service
            .signature_help(&f.main, source.find("1);").expect("value") + 1)
            .expect("private signature")
            .value
            .is_none()
    );
}

#[test]
fn local_non_record_call_targets_do_not_offer_record_field_labels() {
    let source = "program App; type Point = record X: integer; end record; procedure Probe(Point: integer); begin discard Point(X := 1); end procedure; begin end.";
    let mut f = fixture(source);
    assert!(
        f.service
            .definitions(&f.main, source.find("X :=").expect("field"))
            .expect("shadowed label")
            .value
            .is_empty()
    );
    assert!(
        f.service
            .signature_help(&f.main, source.find("1);").expect("value") + 1)
            .expect("shadowed signature")
            .value
            .is_none()
    );
}
