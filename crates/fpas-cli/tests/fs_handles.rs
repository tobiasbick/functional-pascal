//! File handles retain nominal types and errors across cached units and portable images.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures assert process and file outcomes"
)]

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "fpas-file-handles-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("main.fpas"), source).expect("main source");
        fs::write(
            root.join("app.fpasprj"),
            "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n",
        )
        .expect("manifest");
        Self(root)
    }

    fn invoke(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process")
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = self.invoke(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

#[test]
fn file_aliases_modes_and_errors_survive_cached_units_and_source_free_images() {
    let source = r#"program FileImages;
uses FileBridge as Bridge, Std.Io.Fs as Fs, Std.Test as Test, Std.Tasks as Tasks;
function ReadWorker(File: Bridge.File): result of (array of integer, Bridge.FsError);
begin return Bridge.Read(File); end function;
begin
  const File := Bridge.Open('input.bin', Bridge.Mode.Read).Unwrap();
  scope
    const Job := go ReadWorker(File);
    const Bytes := Tasks.Wait(Job).Unwrap();
    Test.AssertEquals(2, Bytes.Length());
    Test.AssertEquals(97, Bytes[0]);
    Test.AssertEquals(98, Bytes[1]);
  end scope;
  Test.AssertEquals(2, Fs.Tell(File).Unwrap());
  Test.AssertEquals(0, Fs.Seek(File, 0, Fs.SeekOrigin.Start).Unwrap());
  Test.AssertEquals('ab', Fs.ReadLine(File).Unwrap().Unwrap());
  Test.AssertTrue(Fs.Close(File).Unwrap());
  case Bridge.Read(File) of
    when Ok(_): Test.Fail('Expected closed file');
    when Error(const Failure):
      Test.AssertTrue(Failure is Fs.FsError.Closed('Std.Io.Fs.Read'));
      Test.AssertTrue(Failure.Message().StartsWith('Std.Io.Fs.FsError.Closed: '));
  end case;
end."#;
    let fixture = Fixture::new(source);
    fs::write(fixture.0.join("input.bin"), b"ab\nlast").expect("input bytes");
    fs::write(
        fixture.0.join("FileBridge.fpas"),
        "unit FileBridge;\nuses Std.Io.Fs as Fs;\npublic type File = Fs.File;\npublic type Mode = Fs.FileMode;\npublic type FsError = Fs.FsError;\npublic function Open(Path: string; Mode: Fs.FileMode): result of (File, FsError);\nbegin return Fs.Open(Path, Mode); end function;\npublic function Read(File: Fs.File): result of (array of integer, FsError);\nbegin return Ok(try Fs.Read(File, 2)); end function;\nend unit;\n",
    )
    .expect("bridge source");

    fixture.run(&["run", "app.fpasprj"]);
    let cache = fixture.0.join("FileBridge.fpascu");
    let previous = fs::read(&cache).expect("compiled bridge");
    let modified = fs::metadata(&cache)
        .expect("bridge metadata")
        .modified()
        .expect("bridge modification time");
    fs::write(
        fixture.0.join("main.fpas"),
        format!("{source}\n// Recompile caller while reusing its unit.\n"),
    )
    .expect("changed caller");
    fixture.run(&["run", "app.fpasprj"]);
    assert_eq!(previous, fs::read(&cache).expect("cached bridge"));
    assert_eq!(
        modified,
        fs::metadata(&cache)
            .expect("bridge metadata")
            .modified()
            .expect("bridge modification time")
    );

    for file in [
        "main.fpas",
        "FileBridge.fpas",
        "FileBridge.fpascu",
        "FileBridge.fpascu.lock",
        "app.fpascp.lock",
    ] {
        let path = fixture.0.join(file);
        if path.exists() {
            fs::remove_file(path).expect("remove source and sidecars");
        }
    }
    fixture.run(&["run", "--std-lib", "absent-library", "app.fpascp"]);
}

#[test]
fn seeded_debugging_rejects_file_creation_before_host_dispatch() {
    let fixture = Fixture::new(
        "program FileEffect; uses Std.Io.Fs as Fs; begin const File := Fs.Open('created.bin', Fs.FileMode.Create).Unwrap(); discard Fs.Close(File).Unwrap(); end.",
    );
    let rejected = fixture.invoke(&["debug", "app.fpasprj", "--schedule-seed", "42"]);
    assert!(!rejected.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&rejected.stdout).expect("seeded debugger JSON");
    assert_eq!(report["schedule"]["status"], "rejected");
    assert!(!fixture.0.join("created.bin").exists());

    let completed = fixture.run(&["debug", "app.fpasprj"]);
    let report: serde_json::Value =
        serde_json::from_slice(&completed.stdout).expect("ordinary debugger JSON");
    assert_eq!(report["outcome"], "completed");
    assert!(fixture.0.join("created.bin").is_file());
}
