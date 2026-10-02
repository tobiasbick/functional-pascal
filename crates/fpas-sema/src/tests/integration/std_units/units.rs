use super::check_errors;

#[test]
fn parse_std_qualified_call_rejects_empty_segments() {
    assert!(crate::std_units::parse_std_qualified_call("Std.Console.WriteLn").is_some());
    assert!(crate::std_units::parse_std_qualified_call("Std..Console.WriteLn").is_none());
    assert!(crate::std_units::parse_std_qualified_call("Std.Console..WriteLn").is_none());
}

#[test]
fn uses_unknown_unit_rejected() {
    let errs = check_errors(
        r#"program T;
uses Foo.Bar as Bar;
begin null;
end program;"#,
    );
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown unit")),
        "{errs:#?}"
    );
}

#[test]
fn uses_bare_std_reserved() {
    let errs = check_errors(
        r#"program T;
uses Std as Std;
begin null;
end program;"#,
    );
    assert!(
        errs.iter().any(|e| e.message.contains("reserved")),
        "{errs:#?}"
    );
}

#[test]
fn uses_std_extra_segment_rejected() {
    let errs = check_errors(
        r#"program T;
uses Std.Console.Extra as Extra;
begin null;
end program;"#,
    );
    assert!(
        errs.iter()
            .any(|e| e.message.contains("reserved namespace `Std`")),
        "{errs:#?}"
    );
}
