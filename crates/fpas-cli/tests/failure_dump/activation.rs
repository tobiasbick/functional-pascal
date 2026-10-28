use super::*;

#[test]
fn all_run_inputs_share_the_report_path_and_preserve_arguments_and_diagnostics() {
    let fixture = Fixture::new();
    fixture.project("program Main; uses Std.Os.Args as Args, Std.Console as Console; begin Console.WriteLn(Args.ParamStr(0)); const Total:=9223372036854775807; panic('boom'); end.");
    fixture.write(
        "suite.fpasworkspace",
        "[workspace]\nname = 'suite'\nmembers = ['app.fpasprj']\n",
    );
    let built = fixture.run(&["build", "app.fpasprj"]);
    exit(&built, 0);
    let baseline = fixture.run(&[
        "run",
        "--diagnostics",
        "json",
        "src/main.fpas",
        "--",
        "--failure-dump",
    ]);
    exit(&baseline, 2);
    for (index, input) in [
        Some("src/main.fpas"),
        Some("app.fpasprj"),
        Some("suite.fpasworkspace"),
        Some("app.fpascp"),
        None,
    ]
    .into_iter()
    .enumerate()
    {
        let report = format!("reports/{index}/failure.json");
        let mut args = vec!["run", "--diagnostics", "json", "--failure-dump", &report];
        if let Some(input) = input {
            args.push(input);
        }
        args.extend(["--", "--failure-dump"]);
        let output = fixture.run(&args);
        exit(&output, 2);
        assert_eq!(output.stdout, baseline.stdout);
        assert_eq!(output.stdout, b"--failure-dump\n");
        let diagnostics = records(&output);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0]["code"], records(&baseline)[0]["code"]);
        let data = fixture.report(&report);
        assert_eq!(data["schema_version"], 1);
        assert_eq!(data["error"]["message"], "panic: boom");
        assert_eq!(data["error"]["task_id"], 0);
        assert_eq!(data["tasks"][0]["observation"], "fault");
        assert_eq!(data["tasks"][0]["inspection"], "available");
        assert_eq!(data["capture"]["incomplete"], false, "{data}");
        for source in data["sources"].as_array().expect("sources") {
            let path = source["path"].as_str().expect("known portable path");
            assert!(!Path::new(path).is_absolute());
            assert!(!path.contains('\\'));
            assert!(!path.contains(fixture.0.to_str().expect("fixture path")));
        }
    }
    fixture.no_staging();
}

#[test]
fn success_preexecution_errors_and_disabled_runs_create_no_report_parents() {
    let fixture = Fixture::new();
    fixture.write(
        "ok.fpas",
        "program Main; begin const Data:result of (integer,string):=Error('data'); end.",
    );
    fixture.write("bad.fpas", "program Main; begin Missing(); end.");
    fixture.write("bad.fpascp", "not an image");
    for (file, code) in [
        ("ok.fpas", 0),
        ("bad.fpas", 1),
        ("bad.fpascp", 1),
        ("missing.fpas", 1),
    ] {
        let output = fixture.run(&["run", file, "--failure-dump", "absent/report.json"]);
        exit(&output, code);
        assert!(!fixture.0.join("absent").exists());
    }
    fixture.write("failed.fpas", "program Main; begin panic('disabled'); end.");
    exit(&fixture.run(&["run", "failed.fpas"]), 2);
    fixture.write("handled.fpas", "program Handled; uses Std.Tasks as Tasks; begin const Group:=Tasks.CreateTaskGroup(); const Ready:channel of integer:=Tasks.CreateChannel(1); discard Tasks.StartTaskInGroup(Group, procedure(Token:Tasks.CancellationToken) begin discard Tasks.Send(Ready,1); panic('handled'); end procedure); discard Tasks.Receive(Ready); discard Tasks.CloseTaskGroup(Group); end.");
    exit(
        &fixture.run(&[
            "run",
            "handled.fpas",
            "--failure-dump",
            "absent/report.json",
        ]),
        0,
    );
    assert!(!fixture.0.join("absent").exists());
    fixture.no_staging();
}

#[test]
fn activation_is_run_only_and_rejects_invalid_or_existing_destinations_before_execution() {
    let fixture = Fixture::new();
    fixture.write("main.fpas", "program Main; uses Std.Console as Console; begin Console.WriteLn('executed'); panic('fault'); end.");
    fixture.write("existing.json", "preserved");
    fs::create_dir(fixture.0.join("directory")).expect("existing directory");
    for mut args in [
        vec!["run", "main.fpas", "--failure-dump"],
        vec!["run", "main.fpas", "--failure-dump", ""],
        vec!["run", "main.fpas", "--failure-dump", "-"],
        vec!["run", "main.fpas", "--failure-dump", "existing.json"],
        vec!["run", "main.fpas", "--failure-dump", "directory"],
        vec![
            "run",
            "main.fpas",
            "--failure-dump",
            "one.json",
            "--failure-dump",
            "two.json",
        ],
    ] {
        args.extend(["--diagnostics", "json"]);
        let output = fixture.run(&args);
        exit(&output, 1);
        assert!(output.stdout.is_empty());
        assert_eq!(records(&output)[0]["code"], "FP4137");
    }
    for command in ["check", "build", "debug", "test", "fmt", "init", "env"] {
        let output = fixture.run(&[command, "--failure-dump", "failure.json"]);
        exit(&output, 1);
        if command == "debug" {
            assert!(output.stderr.is_empty());
            let report: Value = serde_json::from_slice(&output.stdout).expect("batch error JSON");
            assert_eq!(report["diagnostics"][0]["code"], "FP4137");
        } else {
            assert!(String::from_utf8_lossy(&output.stderr).contains("FP4137"));
        }
    }
    assert_eq!(
        fs::read_to_string(fixture.0.join("existing.json")).expect("original"),
        "preserved"
    );
    assert!(!fixture.0.join("failure.json").exists());
    fixture.no_staging();
}
