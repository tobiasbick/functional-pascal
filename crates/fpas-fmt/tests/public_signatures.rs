//! Public view content at the formatter's source-declaration boundary.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "fixtures have explicit assertions"
)]

use fpas_fmt::{FormatError, format_public_signatures};
use fpas_parser::{CompilationUnit, parse_compilation_unit};

#[test]
fn view_keeps_public_type_contracts_and_hides_implementation_details() {
    let source = r#"unit Demo.Api;
uses Other.Types as Types;
const Hidden: integer := 9;
public const Limit: integer := Hidden;
public var Current: integer := Hidden;
public type Id = distinct integer;
public type Range = 1..9;
public type Reply = enum
  Empty;
  Value(Item: Types.Item);
end enum;
public type Box of T = record
  Secret: integer := 7;
  public Item: option of T := None;
  public function Read(Self: Box of T): option of T;
  begin return Self.Item; end function;
  public static procedure Touch(var Value: integer);
  begin Value := Value + 1; end procedure;
  function HiddenRead(Self: Box of T): integer;
  begin return Self.Secret; end function;
end record;
public function Echo<T: Numeric>(Value: T): T;
requires Value = Value;
ensures (Returned): Returned = Value;
begin return Value; end function;
end unit;
"#;
    let (parsed, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("unit fixture");
    };
    let view = format_public_signatures(source, &unit).expect("public view");
    assert_eq!(
        view,
        "unit Demo.Api;\n\nuses Other.Types as Types;\n\npublic const Limit: integer;\n\npublic var Current: integer;\n\npublic type Id = distinct integer;\n\npublic type Range = 1..9;\n\npublic type Reply = enum\n  Empty;\n  Value(Item: Types.Item);\nend enum;\n\npublic type Box of T = record\n  public Item: option of T := None;\n\n  public function Read(Self: Box of T): option of T;\n\n  public static procedure Touch(var Value: integer);\nend record;\n\npublic function Echo<T: Numeric>(Value: T): T;\nrequires Value = Value;\nensures (Returned): Returned = Value;\n\nend unit;\n"
    );
}

#[test]
fn public_view_rejects_a_different_source_ast() {
    let source = "unit Demo;\npublic const Value: integer := 1;\nend unit;\n";
    let (parsed, _) = parse_compilation_unit(source);
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("unit fixture");
    };
    assert_eq!(
        format_public_signatures(&source.replace("Value", "Other"), &unit),
        Err(FormatError::SourceMismatch)
    );
}
