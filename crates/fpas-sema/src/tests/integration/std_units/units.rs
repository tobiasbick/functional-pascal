use super::check_errors;

#[test]
fn parse_std_qualified_call_rejects_empty_segments() {
    assert!(crate::std_units::parse_std_qualified_call("Std.Console.WriteLn").is_some());
    assert!(crate::std_units::parse_std_qualified_call("Std..Console.WriteLn").is_none());
    assert!(crate::std_units::parse_std_qualified_call("Std.Console..WriteLn").is_none());
}

#[test]
fn uses_unknown_unit_rejected() {
    let errs = check_errors("program T;\nuses Foo.Bar as Bar;\nbegin\nend.");
    assert!(
        errs.iter().any(|e| e.message.contains("Unknown unit")),
        "{errs:#?}"
    );
}

#[test]
fn uses_bare_std_reserved() {
    let errs = check_errors("program T;\nuses Std as StdUnit;\nbegin\nend.");
    assert!(
        errs.iter().any(|e| e.message.contains("reserved")),
        "{errs:#?}"
    );
}

#[test]
fn uses_std_extra_segment_rejected() {
    let errs = check_errors("program T;\nuses Std.Console.Extra as Extra;\nbegin\nend.");
    assert!(
        errs.iter()
            .any(|e| e.message.contains("reserved namespace `Std`")),
        "{errs:#?}"
    );
}
