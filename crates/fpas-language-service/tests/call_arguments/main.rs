//! Signature help and navigation for named reference arguments.

#[path = "../support/mod.rs"]
mod support;

use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

const SOURCE: &str = "program NamedVars;
procedure Increase(var Value: integer; Step: integer);
begin Value := Value + Step; end procedure;
begin
  var Value: integer := 0;
  Increase(Step := 1, Value := var Value);
end.
";

#[test]
fn signature_help_selects_reordered_named_var_parameters() {
    let temp = TempDirectory::new("named-var-signatures");
    let path = temp.write("named.fpas", SOURCE);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (marker, expected) in [("Step := 1", 1), ("Value := var Value", 0)] {
        let cursor = SOURCE.find(marker).expect("named argument") + marker.len();
        let help = service
            .signature_help(&path, cursor)
            .expect("signature help")
            .value
            .expect("callable signature");
        assert_eq!(help.active_parameter, Some(expected));
        assert_eq!(
            help.signature.parameters,
            ["var Value: integer", "Step: integer"]
        );
    }
}

#[test]
fn named_var_labels_and_storage_references_rename_independently() {
    let temp = TempDirectory::new("named-var-rename");
    let path = temp.write("named.fpas", SOURCE);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let parameter = SOURCE.find("Value: integer;").expect("parameter");
    let label = SOURCE.find("Value := var Value").expect("named label");
    let mut offsets = service
        .rename(&path, parameter, "Target")
        .expect("parameter rename")
        .value
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    assert_eq!(
        offsets,
        [
            parameter,
            SOURCE.find("Value := Value + Step").expect("assignment"),
            SOURCE.find("Value + Step").expect("read"),
            label
        ]
    );
    let local = SOURCE.find("Value: integer := 0").expect("local");
    let storage = label + "Value := var ".len();
    let mut offsets = service
        .rename(&path, local, "Counter")
        .expect("storage rename")
        .value
        .iter()
        .map(|edit| edit.range.offset())
        .collect::<Vec<_>>();
    offsets.sort_unstable();
    assert_eq!(offsets, [local, storage]);
}
