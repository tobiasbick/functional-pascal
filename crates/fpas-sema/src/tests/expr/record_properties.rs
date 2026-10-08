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
      return Self.Value;
    end function;
    procedure SetValue(Self: Box; V: integer);
    begin
    end procedure;
    property ValueProp: integer read GetValue write SetValue;
  end record;
begin
  const B: Box := Box( Value := 1 );
  const X: integer := B.ValueProp;
  B.ValueProp := 2;
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
    end procedure;
    property Label: string write SetLabel;
  end record;
begin
  const H: Handle := Handle( Id := 1 );
  H.Label := 'ok';
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
    end procedure;
    property Password: string write SetPassword;
  end record;
begin
  const B: Box := Box( );
  const S: string := B.Password;
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
      return 0;
    end function;
    property Width: integer read GetWidth;
  end record;
begin
  const B: Box := Box( );
  B.Width := 1;
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
      return Self.Text;
    end function;
    property Text: string read GetText;
  end record;
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
    let src = "\
program T;
type
  Box = record
    property Width: integer;
  end record;
begin
end.";
    let (_program, parse_errors) = fpas_parser::parse(src);
    assert!(
        parse_errors.iter().any(|diagnostic| {
            diagnostic
                .as_diagnostic()
                .message
                .contains("at least one of `read` or `write`")
        }),
        "{parse_errors:#?}"
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
      return 0;
    end function;
    property Width: integer read GetWidth;
  end record;
begin
  const B: Box := Box( );
  const W: integer := B.Width;
end.";
    let (program, parse_errors) = fpas_parser::parse(src);
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let metadata = analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert!(
        !metadata.property_reads.is_empty(),
        "expected property read metadata"
    );
    assert!(
        metadata
            .property_reads
            .values()
            .flatten()
            .any(|info| info.getter_name.eq_ignore_ascii_case("Box.GetWidth")),
        "{:#?}",
        metadata.property_reads
    );
}

#[test]
fn property_accessors_allow_local_parameter_copies() {
    check_ok(
        "\
program T;
type
  Box = record
    function GetValue(Self: Box): integer;
    begin
      var LocalSelf: Box := Self;\nreturn 0;
    end function;
    procedure SetValue(Self: Box; Value: integer);
    begin
    end procedure;
    property Value: integer read GetValue write SetValue;
  end record;
begin
end.",
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
      return 0;
    end function;
    property Value: integer read GetValue;
  end record;
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
      return 0;
    end function;
    property Value: integer read GetValue;
  end record;
begin
  const B: Box := Box( Value := 1 );
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
      return 0;
    end function;
    property Value: integer read GetValue;
  end record;
begin
  const B: Box := Box( );
  const C: Box := B with Value := 1; end with;
end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("cannot be set in a `with` update")),
        "{errors:#?}"
    );
}
