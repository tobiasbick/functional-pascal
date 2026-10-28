//! Pure Gregorian values retain methods, privacy and errors across portable artifacts.
#![allow(
    clippy::expect_used,
    reason = "integration fixtures assert calendar outcomes"
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
            "fpas-calendar-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("main.fpas"), source).expect("main source");
        fs::write(root.join("app.fpasprj"),
            "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
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

    fn write_bridge(&self) {
        fs::write(self.0.join("CalendarBridge.fpas"), BRIDGE).expect("bridge source");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("fixture cleanup");
    }
}

const BRIDGE: &str = r#"unit CalendarBridge;
uses Std.Calendar as Cal;
public type Date = Cal.Date;
public type DateTime = Cal.DateTime;
public type CalendarError = Cal.CalendarError;
public function MakeDate(Year: integer; Month: integer; Day: integer): result of (Date, CalendarError);
begin return Ok(try Cal.CreateDate(Year, Month, Day)); end function;
public function Parse(Text: string): result of (DateTime, CalendarError);
begin return Cal.ParseDateTime(Text); end function;
public function DateError(): CalendarError;
begin return Cal.CalendarError.InvalidDate(1900, 2, 29); end function;
public function SyntaxError(): CalendarError;
begin return Cal.CalendarError.InvalidSyntax(4, 'expected delimiter'); end function;
end unit;"#;

const PROGRAM: &str = r#"program CalendarImages;
uses CalendarBridge as Bridge, Std.Calendar as Cal, Std.Test as Test;
procedure CheckUtc(Text: string; Expected: string; Timestamp: integer);
begin
  const Parsed: Bridge.DateTime := Bridge.Parse(Text).Unwrap();
  Test.AssertEquals(Expected, Cal.FormatDateTime(Parsed));
  Test.AssertEquals(Timestamp, Cal.ToTimestampMillis(Parsed));
  Test.AssertEquals(Expected, Cal.FormatDateTime(Cal.FromTimestampMillis(Timestamp).Unwrap()));
end procedure;
function ErrorKind(Failure: Bridge.CalendarError): integer;
begin
  case Failure of
    when Cal.CalendarError.InvalidDate(_, _, _): return 0;
    when Cal.CalendarError.InvalidTime(_, _, _, _): return 1;
    when Cal.CalendarError.InvalidSyntax(_, _): return 2;
    when Cal.CalendarError.InvalidOffset(_): return 3;
    when Cal.CalendarError.OutOfRange(_): return 4;
  end case;
end function;
begin
  const Leap: Bridge.Date := Bridge.MakeDate(2000, 2, 29).Unwrap();
  Test.AssertEquals(2000, Leap.Year());
  Test.AssertEquals(2, Leap.Month());
  Test.AssertEquals(29, Leap.Day());
  Test.AssertEquals('2000-02-29', Cal.FormatDate(Leap));
  Test.AssertEquals('2000-03-01', Cal.FormatDate(Cal.AddDays(Leap, 1).Unwrap()));
  Test.AssertEquals('2000-02-28', Cal.FormatDate(Cal.AddDays(Leap, -1).Unwrap()));
  const March := Cal.ParseDate('2000-03-01').Unwrap();
  Test.AssertEquals(1, Cal.DaysBetween(Leap, March));
  Test.AssertEquals(-1, Cal.DaysBetween(March, Leap));
  const Stamp := Cal.CreateDateTime(2000, 2, 29, 12, 34, 56, 789).Unwrap();
  Test.AssertEquals(2000, Stamp.ToDate().Year());
  Test.AssertEquals(2, Stamp.ToDate().Month());
  Test.AssertEquals(29, Stamp.ToDate().Day());
  Test.AssertEquals(12, Stamp.Hour());
  Test.AssertEquals(34, Stamp.Minute());
  Test.AssertEquals(56, Stamp.Second());
  Test.AssertEquals(789, Stamp.Millisecond());
  const Earlier := Cal.AddMillis(Stamp, -790).Unwrap();
  Test.AssertEquals('2000-02-29T12:34:55.999Z', Cal.FormatDateTime(Earlier));
  Test.AssertEquals(790, Cal.MillisBetween(Earlier, Stamp));
  Test.AssertEquals(-790, Cal.MillisBetween(Stamp, Earlier));
  const Epoch := Cal.FromTimestampMillis(0).Unwrap();
  Test.AssertEquals('1969-12-31T23:59:59.999Z', Cal.FormatDateTime(Cal.AddMillis(Epoch, -1).Unwrap()));
  Test.AssertTrue(Bridge.DateError() is Cal.CalendarError.InvalidDate(1900, 2, 29));
  Test.AssertTrue(Bridge.SyntaxError() is Cal.CalendarError.InvalidSyntax(4, 'expected delimiter'));
  Test.AssertTrue(Bridge.DateError().Message().Length() > 0);
  Test.AssertEquals(0, ErrorKind(Bridge.DateError()));
  case Bridge.MakeDate(1900, 2, 29) of
    when Ok(_): Test.Fail('1900 is not a leap year');
    when Error(const Failure):
      Test.AssertTrue(Failure is Cal.CalendarError.InvalidDate(1900, 2, 29));
  end case;
  case Cal.CreateDateTime(2000, 1, 1, 24, 0, 0, 0) of
    when Ok(_): Test.Fail('Hour 24 is invalid');
    when Error(const Failure):
      Test.AssertEquals(1, ErrorKind(Failure));
      Test.AssertTrue(Failure is Cal.CalendarError.InvalidTime(24, 0, 0, 0));
  end case;
  case Cal.ParseDate('2000/02/29') of
    when Ok(_): Test.Fail('Slash is not a date delimiter');
    when Error(const Failure): Test.AssertEquals(2, ErrorKind(Failure));
  end case;
  case Cal.ParseDateTime('2000-01-01T00:00:00+24:00') of
    when Ok(_): Test.Fail('Offset 24 is invalid');
    when Error(const Failure): Test.AssertEquals(3, ErrorKind(Failure));
  end case;
  case Cal.AddDays(Cal.CreateDate(9999, 12, 31).Unwrap(), 1) of
    when Ok(_): Test.Fail('Date exceeds supported range');
    when Error(const Failure):
      Test.AssertEquals(4, ErrorKind(Failure));
      Test.AssertTrue(Failure is Cal.CalendarError.OutOfRange('Std.Calendar.AddDays'));
  end case;
  CheckUtc('0001-01-01T00:00:00.000Z', '0001-01-01T00:00:00.000Z', -62135596800000);
  CheckUtc('9999-12-31T23:59:59.999Z', '9999-12-31T23:59:59.999Z', 253402300799999);
  CheckUtc('1969-12-31T23:59:59.999Z', '1969-12-31T23:59:59.999Z', -1);
  CheckUtc('1900-03-01T00:00:00.000Z', '1900-03-01T00:00:00.000Z', -2203891200000);
  CheckUtc('2000-02-29T12:34:56.789Z', '2000-02-29T12:34:56.789Z', 951827696789);
  CheckUtc('1999-12-31T23:30:00.000-01:00', '2000-01-01T00:30:00.000Z', 946686600000);
  CheckUtc('2024-02-29T23:15:10.250+02:30', '2024-02-29T20:45:10.250Z', 1709239510250);
end."#;

#[test]
fn calendar_alias_methods_errors_and_utc_vectors_survive_cached_units_and_source_free_images() {
    // UTC expectations were independently computed with Python datetime and
    // integer timedelta arithmetic, including Gregorian century boundaries.
    let fixture = Fixture::new(PROGRAM);
    fixture.write_bridge();
    fixture.run(&["run", "app.fpasprj"]);
    let cache = fixture.0.join("CalendarBridge.fpascu");
    let bytes = fs::read(&cache).expect("compiled bridge");
    let modified = fs::metadata(&cache)
        .expect("metadata")
        .modified()
        .expect("mtime");
    fs::write(
        fixture.0.join("main.fpas"),
        format!("{PROGRAM}\n// Reuse the unchanged calendar bridge.\n"),
    )
    .expect("changed caller");
    fixture.run(&["run", "app.fpasprj"]);
    assert_eq!(bytes, fs::read(&cache).expect("cached bridge"));
    assert_eq!(
        modified,
        fs::metadata(&cache)
            .expect("metadata")
            .modified()
            .expect("mtime")
    );
    for file in [
        "main.fpas",
        "CalendarBridge.fpas",
        "CalendarBridge.fpascu",
        "CalendarBridge.fpascu.lock",
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
fn pure_calendar_is_allowed_by_seeded_debugging_and_recording_capture() {
    let fixture = Fixture::new(PROGRAM);
    fixture.write_bridge();
    let output = fixture.run(&["debug", "app.fpasprj", "--schedule-seed", "42"]);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["outcome"], "completed");
    assert_eq!(report["schedule"]["status"], "controlled");

    // Exercise recording on the same compiler-produced portable image. A newly
    // introduced hosted intrinsic would reject execution while capturing.
    fixture.run(&["build", "app.fpasprj"]);
    let image =
        fpas_program::decode(&fs::read(fixture.0.join("app.fpascp")).expect("program image"))
            .expect("decode image");
    let mut session = fpas_runtime::execution::DebugSession::new(image.into_executable())
        .expect("recording session");
    session.start_recording();
    assert!(matches!(
        session
            .continue_execution()
            .expect("pure execution while recording"),
        fpas_runtime::execution::DebugRunResult::Terminated(_)
    ));
    assert!(!session.recording_truncated());
}

#[test]
fn private_calendar_representation_is_rejected_through_source_and_cached_aliases() {
    let fixture = Fixture::new(
        "program Warm; uses CalendarBridge as Bridge; begin discard Bridge.MakeDate(2000, 2, 29).Unwrap().Year(); end.",
    );
    fixture.write_bridge();
    fixture.run(&["run", "app.fpasprj"]);
    let cache = fixture.0.join("CalendarBridge.fpascu");
    let original = fs::read(&cache).expect("compiled bridge");
    let modified = fs::metadata(&cache)
        .expect("metadata")
        .modified()
        .expect("mtime");
    for (imports, qualifier) in [
        ("uses Std.Calendar as Cal;", "Cal"),
        ("uses CalendarBridge as Bridge;", "Bridge"),
    ] {
        for statement in [
            format!("discard {qualifier}.Date();"),
            format!("discard {qualifier}.DateTime();"),
            format!(
                "discard {qualifier}.Date(YearValue := 2000, MonthValue := 2, DayValue := 29);"
            ),
            format!(
                "const Value := {}(2000, 2, 29).Unwrap(); discard Value.YearValue;",
                if qualifier == "Cal" {
                    "Cal.CreateDate"
                } else {
                    "Bridge.MakeDate"
                }
            ),
            format!(
                "const Value := {}(2000, 2, 29).Unwrap(); discard Value with YearValue := 1900; end with;",
                if qualifier == "Cal" {
                    "Cal.CreateDate"
                } else {
                    "Bridge.MakeDate"
                }
            ),
        ] {
            fs::write(
                fixture.0.join("main.fpas"),
                format!("program Invalid; {imports} begin {statement} end."),
            )
            .expect("private access probe");
            let rejected = fixture.invoke(&["check", "app.fpasprj"]);
            assert!(
                !rejected.status.success(),
                "private operation was accepted: {statement}"
            );
            let diagnostics = String::from_utf8_lossy(&rejected.stderr);
            assert!(
                diagnostics.to_ascii_lowercase().contains("private"),
                "{statement}: {diagnostics}"
            );
        }
    }
    assert_eq!(
        original,
        fs::read(&cache).expect("unchanged cached interface")
    );
    assert_eq!(
        modified,
        fs::metadata(&cache)
            .expect("metadata")
            .modified()
            .expect("mtime")
    );
}
