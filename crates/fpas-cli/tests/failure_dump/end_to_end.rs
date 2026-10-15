//! One real worker fault across compiler, scheduler, CLI and serialized frame values.

use super::*;

#[test]
fn worker_fault_report_keeps_nested_parameters_captures_globals_and_main_identity() {
    let fixture = Fixture::new();
    fixture.write(
        "main.fpas",
        r#"program WorkerReport;
uses Std.Tasks as Tasks;
var Global: integer := 7;
function Worker(Captured: integer; Offset: integer): integer;
  function Leaf(Parameter: integer): integer;
  begin
    const Local := Parameter + Captured;
    panic('worker origin');
  end function;
  function Middle(): integer;
  begin
    const Caller := 23;
    const Value := Leaf(7);
    return Value + Caller;
  end function;
begin
  return Middle() + Offset;
end function;
function Entry(Captured: integer; Offset: integer): integer;
begin
  return Worker(Captured, Offset);
end function;
begin
  Global := 31;
  scope
    const Child := go Entry(11, 5);
    discard Tasks.Wait(Child);
  end scope;
end.
"#,
    );
    let output = fixture.run(&[
        "run",
        "main.fpas",
        "--diagnostics",
        "json",
        "--failure-dump",
        "report.json",
    ]);
    exit(&output, 2);
    let diagnostics = records(&output);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["message"], "panic: worker origin");
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], diagnostics[0]["message"]);
    assert_eq!(report["error"]["location"]["line"], 8);
    assert_eq!(report["related_errors"], serde_json::json!([]));
    assert_eq!(report["capture"]["incomplete"], false, "{report}");
    assert_eq!(report["capture"]["total_tasks"], 2);
    assert_eq!(report["capture"]["omitted_tasks"], 0);
    let cause = report["error"]["task_id"].as_u64().expect("worker origin");
    assert_ne!(
        cause, 0,
        "propagating Wait must not replace the original worker"
    );
    let tasks = report["tasks"].as_array().expect("tasks");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"], 0);
    assert_eq!(tasks[0]["state"], "failed");
    assert_eq!(tasks[0]["observation"], "failure_handling");
    assert_eq!(tasks[1]["id"], cause);
    assert_eq!(
        tasks[1]["name"], "entry",
        "tail calls must preserve the original task name"
    );
    assert_eq!(tasks[1]["state"], "failed");
    assert_eq!(tasks[1]["observation"], "fault");
    let frames = tasks[1]["frames"]
        .as_array()
        .expect("original worker frames");
    assert_eq!(frames.len(), 3);
    for (depth, name, kind, scalar) in [
        (0, "Parameter", "parameter", "7"),
        (0, "Captured", "capture", "11"),
        (0, "Local", "local", "18"),
        (1, "Caller", "local", "23"),
        (2, "Offset", "parameter", "5"),
    ] {
        assert_eq!(frames[depth]["depth"], depth);
        let binding = frames[depth]["bindings"]
            .as_array()
            .expect("bindings")
            .iter()
            .find(|binding| {
                binding["name"]
                    .as_str()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
            })
            .unwrap_or_else(|| panic!("missing {name} in {}", frames[depth]));
        assert_eq!(binding["kind"], kind);
        assert_eq!(binding["value"]["scalar"], scalar);
    }
    assert!(
        report["globals"]
            .as_array()
            .expect("globals")
            .iter()
            .any(|global| global["name"] == "Global" && global["value"]["scalar"] == "31")
    );
    fixture.no_staging();
}
