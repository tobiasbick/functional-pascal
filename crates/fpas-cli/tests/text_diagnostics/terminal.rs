//! Real terminal rendering, including the parallel and isolated worker path.

use super::*;
use std::io::Read;
use std::process::Stdio;

fn terminal(fixture: &Fixture, args: &[&str], no_color: Option<&str>) -> (Output, String) {
    let pty = nix::pty::openpty(None, None).expect("pseudoterminal");
    let mut master = fs::File::from(pty.master);
    let mut command = fixture.command(args);
    command.stderr(Stdio::from(fs::File::from(pty.slave)));
    if let Some(value) = no_color {
        command.env("NO_COLOR", value);
    }
    let output = command.output().expect("terminal CLI starts");
    // Drop the parent-side slave before reading EOF on the master.
    drop(command);
    let mut bytes = Vec::new();
    match master.read_to_end(&mut bytes) {
        Ok(_) => {}
        // Linux reports EIO after the last slave closes; BSD returns EOF.
        Err(error) if error.raw_os_error() == Some(nix::libc::EIO) => {}
        Err(error) => panic!("read terminal: {error}"),
    }
    (
        output,
        String::from_utf8(bytes)
            .expect("terminal UTF-8")
            .replace("\r\n", "\n"),
    )
}

#[test]
fn terminal_color_obeys_no_color_and_keeps_json_plain() {
    let fixture = Fixture::new();
    fixture.write("bad.fpas", "program Main;\nbegin\n  @\nend.\n");
    for (no_color, colored) in [
        (None, true),
        (Some(""), true),
        (Some("0"), false),
        (Some("1"), false),
    ] {
        let (output, text) = terminal(&fixture, &["check", "bad.fpas"], no_color);
        assert_eq!(output.status.code(), Some(1), "{text}");
        assert_eq!(
            text.contains("\x1b[31merror[FP1001]\x1b[0m"),
            colored,
            "{text}"
        );
        assert_eq!(text.contains("\x1b[31m^\x1b[0m"), colored, "{text}");
        assert_excerpt(&text, "  @");
        assert_eq!(text.matches('\x1b').count(), if colored { 4 } else { 0 });
    }
    let (output, json) = terminal(
        &fixture,
        &["check", "--diagnostics", "json", "bad.fpas"],
        None,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!json.contains('\x1b'));
    let record: serde_json::Value = serde_json::from_str(json.trim()).expect("unchanged JSON");
    assert_eq!(record["code"], "FP1001");
    assert_eq!(
        json,
        stderr(&fixture.run(&["check", "--diagnostics", "json", "bad.fpas"])),
        "terminal policy must not change JSON bytes"
    );
}

#[test]
fn terminal_policy_and_excerpt_indentation_survive_parallel_isolated_workers() {
    let fixture = Fixture::new();
    for name in ["one_test.fpas", "two_test.fpas"] {
        fixture.write(name, "program Main;\nbegin\n  panic('worker');\nend.\n");
    }
    for timeout in [false, true] {
        for (no_color, colored) in [(None, true), (Some("0"), false)] {
            let mut args = vec!["test", "--jobs", "2"];
            if timeout {
                args.extend(["--timeout", "10"]);
            }
            args.push(".");
            let (output, text) = terminal(&fixture, &args, no_color);
            assert_eq!(output.status.code(), Some(3), "{text}");
            assert_eq!(text.matches(" |   panic('worker');").count(), 2, "{text}");
            assert_eq!(
                text.matches("\x1b[31merror[FP5010]\x1b[0m").count(),
                if colored { 2 } else { 0 },
                "{text}"
            );
            assert_eq!(
                text.matches('\x1b').count(),
                if colored { 8 } else { 0 },
                "{text}"
            );
            for line in text.lines().filter(|line| line.contains(" | ")) {
                assert!(line.starts_with("    "), "indented diagnostic line: {line}");
            }
        }
    }
}
