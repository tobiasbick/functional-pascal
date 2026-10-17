//! Source identity, stream policy, and worker transport at the real CLI boundary.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "integration fixtures assert process results directly"
)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-text-diagnostics-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        Self(root)
    }

    fn write(&self, name: &str, source: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("source directory");
        fs::write(path, source).expect("fixture source");
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command
            .current_dir(&self.0)
            .args(args)
            .env_remove("NO_COLOR");
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("CLI starts")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

fn assert_excerpt(text: &str, line: &str) {
    assert!(text.contains(&format!(" | {line}\n")), "{text}");
    assert!(
        text.lines()
            .any(|line| line.contains(" | ") && line.contains('^')),
        "{text}"
    );
}

#[test]
fn source_project_workspace_and_import_failures_show_the_matching_line_in_plain_pipes() {
    let fixture = Fixture::new();
    fixture.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/*.fpas']\n");
    fixture.write(
        "app.fpasworkspace",
        "[workspace]\nname = 'app'\nmembers = ['app.fpasprj']\n",
    );
    for (source, line, code) in [
        ("program Main;\nbegin\n  ~\nend.\n", "  ~", "FP1001"),
        ("program Main\nbegin\nend.\n", "begin", "FP2001"),
        (
            "program Main;\nbegin\n  Missing();\nend.\n",
            "  Missing();",
            "FP3003",
        ),
    ] {
        fixture.write("src/main.fpas", source);
        for (command, input) in [
            ("check", "src/main.fpas"),
            ("run", "src/main.fpas"),
            ("build", "app.fpasprj"),
            ("check", "app.fpasworkspace"),
        ] {
            let output = fixture.run(&[command, input]);
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            let text = stderr(&output);
            assert!(text.contains(code), "{text}");
            assert!(!text.contains('\x1b'), "pipe must be plain: {text}");
            assert_excerpt(&text, line);
        }
    }
    fixture.write(
        "src/main.fpas",
        "program Main;\nuses App.Bad as Bad;\nbegin end.\n",
    );
    for bad in [
        "unit App.Bad;\npublic procedure Work();\nbegin ~ end procedure;\nend unit;\n",
        "unit App.Bad;\npublic procedure Work();\nbegin Missing(); end procedure;\nend unit;\n",
    ] {
        fixture.write("src/bad.fpas", bad);
        for command in ["check", "build", "run"] {
            let output = fixture.run(&[command, "app.fpasprj"]);
            let text = stderr(&output);
            assert_eq!(output.status.code(), Some(1), "{text}");
            assert!(text.contains("bad.fpas:3:"), "{text}");
            assert_excerpt(&text, bad.lines().nth(2).expect("bad line"));
            assert!(!text.contains(" | begin end."), "{text}");
        }
    }
}

#[test]
fn runtime_points_use_snapshots_and_images_reject_missing_or_mismatched_sources() {
    let fixture = Fixture::new();
    fixture.write("app.fpasprj", "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n");
    let source = "program Main;\nbegin\n  panic('boom');\nend.\n";
    fixture.write("main.fpas", source);
    for input in ["main.fpas", "app.fpasprj", "app.fpascp"] {
        let output = fixture.run(&["run", input]);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let text = stderr(&output);
        assert_excerpt(&text, "  panic('boom');");
        assert!(text.contains("\n    |   ^\n"), "runtime point: {text}");
    }
    fixture.write(
        "main.fpas",
        "program Other;\nbegin\n  panic('unrelated');\nend.\n",
    );
    for missing in [false, true] {
        if missing {
            fs::remove_file(fixture.0.join("main.fpas")).expect("remove source");
        }
        let output = fixture.run(&["run", "app.fpascp"]);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(2));
        assert!(text.contains("error[FP5010]: panic: boom"), "{text}");
        assert!(text.contains("  help:"), "{text}");
        assert!(!text.contains(" | "), "unverified text: {text}");
    }
    fixture.write("main.fpas", "program Main;\nuses Std.Fs as Fs;\nbegin\n  discard Fs.WriteText('main.fpas', 'replaced').Unwrap();\n  panic('snapshot');\nend.\n");
    let output = fixture.run(&["run", "main.fpas"]);
    let text = stderr(&output);
    assert_eq!(output.status.code(), Some(2), "{text}");
    assert_eq!(
        fs::read_to_string(fixture.0.join("main.fpas")).expect("changed file"),
        "replaced"
    );
    assert_excerpt(&text, "  panic('snapshot');");
}

#[test]
fn precompiled_test_sources_are_not_replaced_by_an_earlier_test() {
    let fixture = Fixture::new();
    let old = "program Main;\nbegin\n  panic('old');\nend.\n";
    let new = "program Main;\nbegin\n  panic('new');\nend.\n";
    let replacement = new.replace('\'', "''").replace('\n', "'#10'");
    fixture.write("00_mutate_test.fpas", &format!("program Change; uses Std.Fs as Fs; begin discard Fs.WriteText('10_panic_test.fpas', '{replacement}').Unwrap(); end."));
    for timeout in [false, true] {
        fixture.write("10_panic_test.fpas", old);
        let mut args = vec!["test", "--jobs", "1"];
        if timeout {
            args.extend(["--timeout", "10"]);
        }
        args.push(".");
        let output = fixture.run(&args);
        let text = stderr(&output);
        assert_eq!(output.status.code(), Some(3), "{text}");
        assert_eq!(
            fs::read_to_string(fixture.0.join("10_panic_test.fpas")).expect("changed source"),
            new
        );
        assert!(
            text.contains("panic: old"),
            "precompiled bytecode must execute: {text}"
        );
        assert_excerpt(&text, "  panic('old');");
        assert!(
            !text.contains(" |   panic('new');"),
            "mismatched source: {text}"
        );
    }
}

#[cfg(unix)]
#[path = "text_diagnostics/terminal.rs"]
mod terminal;
