use super::{check_errors, check_ok};

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

#[test]
fn source_cli_import_does_not_register_intrinsic_symbols() {
    check_ok("program T;\nuses Std.Cli as Cli;\nbegin\nend.");
    assert!(crate::intrinsic_std_symbols(fpas_std::STD_UNIT_CLI).is_empty());
}

#[test]
fn source_calendar_import_does_not_register_intrinsic_symbols() {
    check_ok("program T;\nuses Std.Calendar as Calendar;\nbegin\nend.");
    assert!(crate::intrinsic_std_symbols(fpas_std::STD_UNIT_CALENDAR).is_empty());
}
