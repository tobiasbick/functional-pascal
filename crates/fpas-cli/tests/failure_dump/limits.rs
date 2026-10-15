use super::*;

#[test]
fn byte_heavy_primary_values_preserve_related_errors_and_cause_task_headers() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Secondary; uses Std.Tasks as Tasks; var Items:array of string:=[]; procedure Leaf(Values:array of string); begin const Copy:=Values; panic('body'); end procedure; procedure Caller(); begin const Fill:=(#0).RepeatStr(4096); Leaf(Items); panic('unreachable'); end procedure; begin for I:integer:=1 to 256 do Items.PushInPlace((#0).RepeatStr(4096)); end for; const Ready:channel of integer:=Tasks.CreateChannel(1); const Group:=Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, procedure(Token:Tasks.CancellationToken) begin discard Tasks.Send(Ready,1); panic('child'); end procedure); discard Tasks.Receive(Ready); Caller(); panic('unreachable'); end.");
    exit(
        &fixture.run(&["run", "main.fpas", "--failure-dump", "report.json"]),
        2,
    );
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: body");
    assert_eq!(report["related_errors"][0]["message"], "panic: child");
    let child = report["related_errors"][0]["task_id"]
        .as_u64()
        .expect("child task");
    let tasks = report["tasks"].as_array().expect("task headers");
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"], 0);
    assert_eq!(tasks[1]["id"], child);
    assert_eq!(report["capture"]["omitted_tasks"], 0);
    assert_eq!(report["capture"]["incomplete"], true);
    assert!(
        report["capture"]["reasons"]
            .as_array()
            .expect("reasons")
            .contains(&Value::String("byte_limit".into()))
    );
}

#[test]
fn escaped_boundary_reports_remain_valid_and_disclose_omitted_bindings() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Main; procedure Leaf(Items:array of string); begin const Copy:=Items; panic('bounded'); end procedure; begin var Items:array of string:=[]; for I:integer:=1 to 256 do Items.PushInPlace((#0).RepeatStr(4096)); end for; Leaf(Items); end.");
    let output = fixture.run(&["run", "main.fpas", "--failure-dump", "report.json"]);
    exit(&output, 2);
    let report = fixture.report("report.json");
    assert_eq!(report["error"]["message"], "panic: bounded");
    assert_eq!(report["capture"]["incomplete"], true);
    assert!(
        report["capture"]["reasons"]
            .as_array()
            .expect("reasons")
            .contains(&Value::String("byte_limit".into()))
    );
    assert_eq!(report["tasks"][0]["id"], 0);
    assert_eq!(report["tasks"][0]["inspection"], "partial");
    assert_eq!(report["tasks"][0]["unavailable_reason"], "byte_limit");
    assert_eq!(report["tasks"][0]["frames"][0]["truncated"], true);
    assert!(
        fs::metadata(fixture.0.join("report.json"))
            .expect("report size")
            .len()
            > 6 * 1024 * 1024
    );
    fixture.no_staging();
}
