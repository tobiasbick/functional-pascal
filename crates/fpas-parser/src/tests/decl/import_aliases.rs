//! Contextual aliases, written spelling, exact spans, and malformed-import recovery.

use super::*;

#[test]
fn imports_keep_unit_alias_and_source_spans() {
    let source = "program T; uses Std.Math AS Numbers, App.As as As, Std.Console; begin end.";
    let program = parse_ok(source);
    assert_eq!(program.uses.len(), 3);
    let first = &program.uses[0];
    assert_eq!(first.unit.parts, ["Std", "Math"]);
    let alias = first.alias.as_ref().expect("alias");
    assert_eq!(alias.name, "Numbers");
    assert_eq!(
        &source[alias.span.offset..alias.span.offset + alias.span.length],
        "Numbers"
    );
    assert_eq!(
        &source[first.span.offset..first.span.offset + first.span.length],
        "Std.Math AS Numbers"
    );
    assert_eq!(program.uses[1].unit.parts, ["App", "As"]);
    assert_eq!(program.uses[1].alias.as_ref().expect("alias").name, "As");
    assert!(program.uses[2].alias.is_none());
}

#[test]
fn as_remains_an_identifier_outside_uses() {
    let unit = parse_unit_ok(
        "unit Demo.As; uses App.Api aS Library; public function As(As: integer): integer; begin return As; end function; end unit;",
    );
    assert_eq!(unit.name.parts, ["Demo", "As"]);
    assert_eq!(unit.uses[0].alias.as_ref().expect("alias").name, "Library");
    let program = parse_ok("program As; const As: integer := 1; begin end.");
    assert_eq!(program.name, "As");
}

#[test]
fn missing_alias_recovers_at_comma_and_semicolon() {
    for source in [
        "program T; uses Std.Math as, Std.Console; begin end.",
        "program T; uses Std.Math as; begin end.",
    ] {
        let (program, errors) = parse_with_errors(source);
        assert!(!errors.is_empty());
        assert_eq!(program.name, "T");
        assert!(program.uses[0].alias.is_some());
        assert_eq!(program.uses.len(), if source.contains(',') { 2 } else { 1 });
    }
}
