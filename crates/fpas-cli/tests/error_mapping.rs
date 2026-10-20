//! Constructor values and MapError across source, cached units and runtime routes.
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
            "fpas-map-error-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("fixture directory");
        fs::write(root.join("main.fpas"), source).expect("fixture source");
        Self(root)
    }
    fn run(&self, args: &[&str], code: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("CLI process");
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
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
fn imported_generic_constructor_values_survive_cached_units_and_program_images() {
    let f = Fixture::new(
        r#"program Main;
uses App.Errors as Imported, Std.Test as Test;
type Alias = Imported.Leaf;
begin
  const MakePair: function(Word: string; Count: integer): Imported.Entry := Imported.Entry.Pair;
  Test.AssertTrue(MakePair(Count := 2, Word := 'enum') is Imported.Entry.Pair('enum', 2));
  const MakeBox: function(Text: string): Imported.Box of string := Imported.Box.Item;
  Test.AssertTrue(MakeBox('boxed') is Imported.Box.Item('boxed'));
  const Input: result of (integer, Alias) := Error(Alias.Bad('first'));
  const Wrapped := Input.MapError(Imported.Context.Failed);
  Test.AssertTrue(Wrapped is Error(Imported.Context.Failed(Alias.Bad('first'))));
  const Other: result of (boolean, Imported.Other) := Error(Imported.Other.Bad(2));
  const Second := Imported.Map(Other);
  Test.AssertTrue(Second is Error(Imported.Context.Failed(Imported.Other.Bad(2))));
  const Factory: function(Value: Imported.Leaf): Imported.Context of Imported.Leaf := Imported.Context.Failed;
  case Imported.Map(Input) of
    when Ok(_): Test.AssertTrue(false);
    when Error(const Value): Test.AssertEquals(Factory(Alias.Bad('first')).Message(), Value.Message());
  end case;
end."#,
    );
    fs::write(
        f.0.join("errors.fpas"),
        r#"unit App.Errors;
public type Leaf = error Bad(Text: string); end error;
public type Other = error Bad(Code: integer); end error;
public type Context of E: Error = error Failed(Cause: E); end error;
public type Entry = enum Pair(Text: string; Count: integer); end enum;
public type Box of T = enum Item(Value: T); end enum;
public function Map<T, E: Error>(Value: result of (T, E)): result of (T, Context of E);
begin return Value.MapError(Context.Failed); end function;
end unit;"#,
    )
    .expect("unit");
    fs::write(f.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    f.run(&["run", "app.fpasprj"], 0);
    assert!(f.0.join("errors.fpascu").is_file());
    let cached = fs::read(f.0.join("errors.fpascu")).expect("cached unit");
    f.run(&["run", "app.fpasprj"], 0);
    assert_eq!(
        cached,
        fs::read(f.0.join("errors.fpascu")).expect("cached unit")
    );
    f.run(&["build", "app.fpasprj"], 0);
    f.run(&["run", "app.fpascp"], 0);
}

#[test]
fn map_error_resumes_nested_callbacks_and_preserves_trace_bounds() {
    let f = Fixture::new(
        r#"program Scheduled;
uses Std.Tasks as Tasks, Std.Time as Time, Std.Test as Test;
var Calls: integer := 0;
function Inner(Value: string): integer;
begin Time.Sleep(1); return Value.Length(); end function;
function Outer(Value: string): integer;
begin
  const Input: result of (boolean, string) := Error(Value);
  return case Input.MapError(Inner) of when Ok(_): 0; when Error(const N): N; end case;
end function;
function Work(): integer;
begin
  const Input: result of (boolean, string) := Error('abc');
  return case Input.MapError(Outer) of when Ok(_): 0; when Error(const N): N; end case;
end function;
begin scope
  const Pending := go Work();
  Calls := Tasks.Wait(Pending);
  Test.AssertEquals(3, Calls);
end scope; end."#,
    );
    f.run(&["run", "main.fpas"], 0);
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--schedule-seed",
            "42",
        ],
        0,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], true);
    assert_eq!(report["trace"]["event_count"], 2);
    assert_eq!(report["schedule"]["status"], "controlled");
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--trace-limit",
            "1",
        ],
        3,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], false);
}

#[test]
fn mapper_panics_and_contracts_remain_runtime_failures_on_main_and_tasks() {
    for failure in ["panic('mapper failed');", "return 1;"] {
        for scheduled in [false, true] {
            let contract = if failure.starts_with("return") {
                "requires Value = 'allowed';"
            } else {
                ""
            };
            let invoke = if scheduled {
                "scope const Pending := go Work(); discard Tasks.Wait(Pending); end scope;"
            } else {
                "discard Work();"
            };
            let f = Fixture::new(&format!(
                "program Failure; uses Std.Tasks as Tasks; function Mapper(Value: string): integer; {contract} begin {failure} end function; function Work(): integer; begin const Input: result of (integer, string) := Error('bad'); discard Input.MapError(Mapper); return 0; end function; begin {invoke} end."
            ));
            let output = f.run(&["run", "main.fpas", "--failure-dump", "failure.json"], 2);
            assert!(!output.stderr.is_empty());
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(f.0.join("failure.json")).expect("failure dump"))
                    .expect("failure JSON");
            assert!(report.is_object());
        }
    }
}

#[test]
fn synchronous_mapper_mutations_obey_callback_trace_bounds() {
    let f = Fixture::new(
        r#"program TracedMapper;
var Calls: integer := 0;
function Mapper(Value: string): integer;
begin Calls := Calls + 1; return Value.Length(); end function;
begin
  const Input: result of (boolean, string) := Error('abc');
  discard Input.MapError(Mapper);
end."#,
    );
    let output = f.run(&["debug", "main.fpas", "--trace", "Calls"], 0);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["trace"]["complete"], true);
    assert_eq!(report["trace"]["event_count"], 2);
    assert_eq!(report["trace"]["events"][1]["new_value"]["scalar"], "1");
    let output = f.run(
        &[
            "debug",
            "main.fpas",
            "--trace",
            "Calls",
            "--trace-limit",
            "1",
        ],
        3,
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("debug JSON");
    assert_eq!(report["outcome"], "limit");
    assert_eq!(report["trace"]["complete"], false);
    assert!(
        report["trace"]["reasons"]
            .as_array()
            .expect("trace reasons")
            .contains(&serde_json::Value::from("trace_event_limit"))
    );
}
