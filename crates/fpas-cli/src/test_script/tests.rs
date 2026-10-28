//! Unit tests for test script parsing and application.

use super::{apply_script_to_vm, load_script, parse_script_text, sidecar_path_for_test};
use fpas_compiler::compile;
use fpas_parser::parse;
use std::path::Path;

#[test]
fn sidecar_path_replaces_fpas_extension() {
    let path = Path::new("tests/console/readln_test.fpas");
    assert_eq!(
        sidecar_path_for_test(path),
        Path::new("tests/console/readln_test.script.toml")
    );
}

#[test]
fn parse_readln_event() {
    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "Alice"
"#,
        Path::new("demo.script.toml"),
    )
    .expect("parse");
    assert_eq!(script.events.len(), 1);
}

#[test]
fn parse_unknown_event_type_is_error() {
    let err = parse_script_text(
        r#"
[[event]]
type = "teleport"
"#,
        Path::new("bad.script.toml"),
    )
    .expect_err("unknown event types must be rejected");
    assert!(err.contains("Unknown event type `teleport`"));
}

#[test]
fn malformed_structured_console_events_identify_the_invalid_field() {
    for (event, field) in [
        ("type = 'console_key'", "kind"),
        ("type = 'console_key'\nkind = 'Teleport'", "kind"),
        ("type = 'console_key'\nkind = 'Character'", "ch"),
        ("type = 'console_key'\nkind = 'Character'\nch = ''", "ch"),
        ("type = 'console_key'\nkind = 'Character'\nch = 'ab'", "ch"),
        ("type = 'console_key'\nkind = 'Character'\nch = 1", "ch"),
        ("type = 'console_key'\nkind = 'Escape'\nch = 'q'", "ch"),
        (
            "type = 'console_key'\nkind = 'Escape'\nctrl = 'yes'",
            "ctrl",
        ),
        (
            "type = 'console_key'\nkind = 'Escape'\ncontrol = true",
            "control",
        ),
        ("type = 'console_resize'\nheight = 24", "width"),
        ("type = 'console_resize'\nwidth = 0\nheight = 24", "width"),
        ("type = 'console_resize'\nwidth = -1\nheight = 24", "width"),
        (
            "type = 'console_resize'\nwidth = 65536\nheight = 24",
            "width",
        ),
        (
            "type = 'console_resize'\nwidth = '80'\nheight = 24",
            "width",
        ),
        ("type = 'console_resize'\nwidth = 80\nheight = 0", "height"),
    ] {
        let script = format!("[[event]]\n{event}\n");
        let error = parse_script_text(&script, Path::new("invalid.script.toml"))
            .expect_err("malformed terminal input must fail before execution");
        assert!(error.contains(field), "{event}: {error}");
        assert!(error.contains("invalid.script.toml"), "{error}");
    }
}

#[test]
fn apply_readln_script_runs_readln_test_program() {
    let source = "program T;\nuses Std.Console as Console, Std.Test as Test;\nbegin\n  Test.AssertTrue(Console.ReadLn().Unwrap() = 'Alice');\nend.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_runtime::execution::Vm::new(executable);

    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "Alice"
"#,
        Path::new("readln.script.toml"),
    )
    .expect("parse");
    apply_script_to_vm(&mut vm, &script);
    vm.run().expect("run");
}

#[test]
fn apply_readln_events_are_consumed_in_script_order() {
    let source = "program T;\nuses Std.Console as Console, Std.Test as Test;\nbegin\n  Test.AssertEquals(Console.ReadLn().Unwrap(), 'first');\n  Test.AssertEquals(Console.ReadLn().Unwrap(), 'second');\n  Test.AssertEquals(Console.ReadLn().Unwrap(), 'third');\nend.";
    let (program, _) = parse(source);
    let executable = compile(&program).expect("compile");
    let mut vm = fpas_runtime::execution::Vm::new(executable);

    let script = parse_script_text(
        r#"
[[event]]
type = "readln"
line = "first"

[[event]]
type = "readln"
line = "second"

[[event]]
type = "readln"
line = "third"
"#,
        Path::new("order.script.toml"),
    )
    .expect("parse");
    apply_script_to_vm(&mut vm, &script);
    vm.run().expect("run");
}

#[test]
fn load_script_reads_file_from_disk() {
    let dir = crate::test_support::create_temp_dir("fpas-script-load");
    let path = dir.join("input.script.toml");
    crate::test_support::write_text(&path, "[[event]]\ntype = \"readln\"\nline = \"ok\"\n");
    let script = load_script(&path).expect("load");
    assert_eq!(script.events.len(), 1);
}
