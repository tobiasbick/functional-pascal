//! Real stdin EOF behavior in source projects and portable program images.

#![allow(clippy::expect_used, reason = "process fixtures assert on CLI results")]

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new(body: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "fpas-console-eof-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create fixture");
        fs::write(
            path.join("app.fpasprj"),
            "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['main.fpas']\n",
        )
        .expect("write manifest");
        fs::write(
            path.join("main.fpas"),
            format!(
                "program Input; uses Std.Console as Console, Std.Test as Test; begin {body} Console.WriteLn('ok'); end."
            ),
        )
        .expect("write program");
        Self(path)
    }

    fn command(&self, operation: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_fpas"));
        command.current_dir(&self.0).args([operation, "--std-lib"]);
        command.arg(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../lib")
                .canonicalize()
                .expect("source standard library"),
        );
        command
    }

    fn build(&self) {
        let output = self
            .command("build")
            .arg("app.fpasprj")
            .output()
            .expect("build portable image");
        assert!(output.status.success(), "{output:?}");
        assert!(self.0.join("app.fpascp").is_file());
    }

    fn run(&self, target: &str, input: &[u8]) -> Output {
        let mut child = self
            .command("run")
            .arg(target)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn input fixture");
        child
            .stdin
            .take()
            .expect("piped stdin")
            .write_all(input)
            .expect("write and close input");
        child.wait_with_output().expect("wait for input fixture")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_source_and_image(body: &str, input: &[u8]) {
    let fixture = Fixture::new(body);
    fixture.build();
    for target in ["app.fpasprj", "app.fpascp"] {
        let output = fixture.run(target, input);
        assert!(output.status.success(), "{target}: {output:?}");
        assert_eq!(output.stdout, b"ok\n", "{target}: {output:?}");
    }
}

#[test]
fn readln_distinguishes_empty_lines_final_unterminated_lines_and_eof() {
    assert_source_and_image(
        "Test.AssertTrue(Console.ReadLn() is Some(''));\n\
         Test.AssertTrue(Console.ReadLn() is Some('alpha'));\n\
         Test.AssertTrue(Console.ReadLn() is Some('€tail'));\n\
         Test.AssertTrue(Console.ReadLn() is None);\n\
         Test.AssertTrue(Console.ReadLn() is None);\n\
         Test.AssertTrue(Console.ReadText() is None);",
        "\nalpha\r\n€tail".as_bytes(),
    );
}

#[test]
fn readtext_and_readln_share_normalized_terminators_and_unicode() {
    assert_source_and_image(
        "Test.AssertTrue(Console.ReadText() is Some(#10));\n\
         Test.AssertTrue(Console.ReadText() is Some('€'));\n\
         Test.AssertTrue(Console.ReadLn() is Some('rest'));\n\
         Test.AssertTrue(Console.ReadText() is Some('z'));\n\
         Test.AssertTrue(Console.ReadText() is Some(#13));\n\
         Test.AssertTrue(Console.ReadText() is None);\n\
         Test.AssertTrue(Console.ReadLn() is None);",
        "\r\n€rest\nz\r".as_bytes(),
    );
}

#[test]
fn empty_stdin_is_eof_and_final_lf_does_not_create_another_line() {
    assert_source_and_image(
        "Test.AssertTrue(Console.ReadLn() is None); Test.AssertTrue(Console.ReadText() is None);",
        b"",
    );
    assert_source_and_image(
        "Test.AssertTrue(Console.ReadText() is Some('x'));\n\
         Test.AssertTrue(Console.ReadLn() is Some(''));\n\
         Test.AssertTrue(Console.ReadLn() is None);",
        b"x\n",
    );
}

#[test]
fn invalid_utf8_remains_an_input_failure_in_source_and_images() {
    let fixture = Fixture::new("discard Console.ReadLn();");
    fixture.build();
    for target in ["app.fpasprj", "app.fpascp"] {
        let output = fixture.run(target, &[0xff]);
        assert_eq!(output.status.code(), Some(2), "{target}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("ReadLn failed"));
    }
}
