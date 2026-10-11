//! Clauses and their comments remain stable through canonical formatting.
#[test]
fn contracts_roundtrip_routines_methods_nested_declarations_and_comments()
-> Result<(), fpas_fmt::FormatError> {
    let source = "unit Api;
public procedure P(X: integer); // header
// entry
REQUIRES X>0; // positive
requires if X<10 then true else false end if;
// exit
ENSURES X>0; // current
// helper
procedure Q(); ensures true; begin null; end procedure;
begin Q(); end procedure;
public type Box=record public Value:integer;
public function Read(Self:Box):integer;
requires Self.Value>0;
ensures (Value) Value=Self.Value;
begin return Self.Value; end function;
public static procedure Check(X:integer); requires X>0; ensures X>0; begin null; end procedure;
end record; end unit;";
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let formatted = fpas_fmt::format_source(source, &unit)?;
    for text in [
        "// header",
        "// entry",
        "// positive",
        "// helper",
        "// exit",
        "// current",
        "requires X > 0;",
        "requires Self.Value > 0;",
        "ensures X > 0;",
        "ensures (Value) Value = Self.Value;",
    ] {
        assert!(formatted.contains(text), "{formatted}");
    }
    let (parsed, errors) = fpas_parser::parse_compilation_unit(&formatted);
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(fpas_fmt::format_source(&formatted, &parsed)?, formatted);
    Ok(())
}
