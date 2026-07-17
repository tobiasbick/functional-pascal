//! Record property semantic tests.
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::super::{check_errors, check_ok};
use crate::analyze_with_types;

#[test]
fn read_write_property_ok() {
    check_ok(
        "\
program T;
type
  Box = record
    Value: integer;
    function GetValue(Self: Box): integer;
    begin
      return Self.Value
    end;
    procedure SetValue(Self: Box; V: integer);
    begin
    end;
    property ValueProp: integer read GetValue write SetValue;
  end;
begin
  var B: Box := record Value := 1; end;
  var X: integer := B.ValueProp;
  B.ValueProp := 2
end.",
    );
}

#[test]
fn immutable_binding_can_use_setter() {
    check_ok(
        "\
program T;
type
  Handle = record
    Id: integer;
    procedure SetLabel(Self: Handle; Value: string);
    begin
    end;
    property Label: string write SetLabel;
  end;
begin
  var H: Handle := record Id := 1; end;
  H.Label := 'ok'
end.",
    );
}

#[test]
fn write_only_property_cannot_be_read() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    procedure SetPassword(Self: Box; Value: string);
    begin
    end;
    property Password: string write SetPassword;
  end;
begin
  var B: Box := record end;
  var S: string := B.Password
end.",
    );
    assert!(
        errors.iter().any(|e| e.message.contains("write-only")),
        "{errors:#?}"
    );
}

#[test]
fn read_only_property_cannot_be_written() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    function GetWidth(Self: Box): integer;
    begin
      return 0
    end;
    property Width: integer read GetWidth;
  end;
begin
  var B: Box := record end;
  B.Width := 1
end.",
    );
    assert!(
        errors.iter().any(|e| e.message.contains("read-only")),
        "{errors:#?}"
    );
}

#[test]
fn property_duplicates_field_name() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    Text: string;
    function GetText(Self: Box): string;
    begin
      return Self.Text
    end;
    property Text: string read GetText;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("Duplicate record member")),
        "{errors:#?}"
    );
}

#[test]
fn property_missing_accessor_rejected() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    property Width: integer;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("at least one of `read` or `write`")),
        "{errors:#?}"
    );
}

#[test]
fn property_read_records_metadata() {
    let src = "\
program T;
type
  Box = record
    function GetWidth(Self: Box): integer;
    begin
      return 0
    end;
    property Width: integer read GetWidth;
  end;
begin
  var B: Box := record end;
  var W: integer := B.Width
end.";
    let (program, parse_errors) = fpas_parser::parse(src);
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let (errors, _, _, _, _, _, _, _, reads, _) = analyze_with_types(&program);
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(!reads.is_empty(), "expected property read metadata");
    assert!(
        reads
            .values()
            .flatten()
            .any(|info| info.getter_name.eq_ignore_ascii_case("Box.GetWidth")),
        "{reads:#?}"
    );
}

#[test]
fn property_rejects_mutable_accessor_parameters() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    function GetValue(mutable Self: Box): integer;
    begin
      return 0
    end;
    procedure SetValue(Self: Box; mutable Value: integer);
    begin
    end;
    property Value: integer read GetValue write SetValue;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("mutable Self")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("by value")),
        "{errors:#?}"
    );
}

#[test]
fn property_rejects_generic_accessors() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    function GetValue<T>(Self: Box): integer;
    begin
      return 0
    end;
    property Value: integer read GetValue;
  end;
begin
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be generic")),
        "{errors:#?}"
    );
}

#[test]
fn property_cannot_be_initialized_as_a_record_field() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    function GetValue(Self: Box): integer;
    begin
      return 0
    end;
    property Value: integer read GetValue;
  end;
begin
  var B: Box := record Value := 1; end
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be initialized")),
        "{errors:#?}"
    );
}

#[test]
fn property_cannot_be_used_as_a_record_update_field() {
    let errors = check_errors(
        "\
program T;
type
  Box = record
    function GetValue(Self: Box): integer;
    begin
      return 0
    end;
    property Value: integer read GetValue;
  end;
begin
  var B: Box := record end;
  var C: Box := B with Value := 1; end
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be set in a `with` update")),
        "{errors:#?}"
    );
}
