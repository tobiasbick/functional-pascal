#![allow(
    clippy::expect_used,
    reason = "rename fixtures use explicit source offsets"
)]

mod support;

use fpas_language_service::{LanguageService, RenameError, WorkspaceContext};
use support::TempDirectory;

#[test]
fn rename_rejects_an_inner_declaration_capturing_edited_global_uses() {
    let temp = TempDirectory::new("rename-global-capture");
    let source = r#"program GlobalCapture;

var Source: integer := 1;

function Read(): integer;
begin
  var Captured: integer := 2;
  return Source
end;

begin
  var Result: integer := Read()
end.
"#;
    let path = temp.write("global.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let error = service
        .rename(
            &path,
            source.find("return Source").expect("global use") + "return ".len(),
            "Captured",
        )
        .expect_err("inner declaration would capture the edited use");
    assert_eq!(
        error,
        RenameError::Conflict {
            name: "Captured".to_owned()
        }
    );
}

#[test]
fn rename_rejects_a_local_declaration_capturing_unedited_outer_uses() {
    let temp = TempDirectory::new("rename-local-capture");
    let source = r#"program LocalCapture;

var Outer: integer := 1;

function Read(): integer;
begin
  var Local: integer := 2;
  return Local + Outer
end;

begin
  var Result: integer := Read()
end.
"#;
    let path = temp.write("local.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));

    let error = service
        .rename(
            &path,
            source.find("return Local").expect("local use") + "return ".len(),
            "Outer",
        )
        .expect_err("renamed local would capture the existing outer use");
    assert_eq!(
        error,
        RenameError::Conflict {
            name: "Outer".to_owned()
        }
    );
}

#[test]
fn rename_allows_disjoint_local_names_and_the_edited_source_resolves() {
    let temp = TempDirectory::new("rename-disjoint-scopes");
    let source = r#"program Disjoint;

function First(): integer;
begin
  var Source: integer := 1;
  return Source
end;

function Second(): integer;
begin
  var Target: integer := 2;
  return Target
end;

begin
  var Result: integer := First() + Second()
end.
"#;
    let path = temp.write("disjoint.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let source_use = source.find("return Source").expect("source use") + "return ".len();
    let edits = service
        .rename(&path, source_use, "Target")
        .expect("disjoint local rename")
        .value;
    assert_eq!(edits.len(), 2, "{edits:?}");

    let mut edited = source.to_owned();
    for edit in edits {
        edited.replace_range(edit.range.offset()..edit.range.end(), &edit.new_text);
    }
    service
        .documents_mut()
        .open_document(&path, 1, edited.clone())
        .expect("edited source overlay");
    let renamed_use = edited.find("return Target").expect("renamed use") + "return ".len();
    let definitions = service
        .definitions(&path, renamed_use)
        .expect("renamed definition")
        .value;
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert!(
        definitions[0].symbol.selection_span.offset()
            < edited.find("function Second").expect("second routine")
    );
}
