//! Final integrated user example through the source and portable-image routes.
use super::*;

#[test]
fn complete_configuration_example_preserves_real_and_local_failures_in_images() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/pascal/errors/config_error.fpas");
    let fixture = Fixture::new(&fs::read_to_string(path).expect("complete user example"));
    fs::write(fixture.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    let source = fixture.run(&["run", "app.fpasprj"], 0);
    fixture.run(&["build", "app.fpasprj"], 0);
    let image = fixture.run(&["run", "app.fpascp"], 0);
    for output in [source, image] {
        let text = String::from_utf8(output.stdout).expect("error output UTF-8");
        assert!(text.contains("ConfigErrors.ContextError.Failed: Operation failed"));
        assert!(text.contains("caused by Std.Io.Fs.FsError.FileNotFound"));
        assert!(text.contains("caused by ConfigErrors.ConfigError.MissingSetting"));
    }
}

#[test]
fn debugger_and_failure_capture_keep_two_concrete_generic_causes() {
    let fixture = Fixture::new(
        "program InspectErrors;\ntype FileIssue = error Missing(Path: string); end error;\ntype TextIssue = error Invalid(Line: integer); end error;\ntype Context of E: Error = error Failed(Cause: E; Step: string); end error;\nvar First: Context of FileIssue := Context.Failed(FileIssue.Missing('settings'), 'load');\nvar Second: Context of TextIssue := Context.Failed(TextIssue.Invalid(3), 'parse');\nbegin\n  panic('capture typed causes');\nend.",
    );
    fs::write(fixture.0.join("app.fpasprj"), "[project]\nname = 'app'\nkind = 'program'\nmain = 'main.fpas'\n[sources]\ninclude = ['*.fpas']\n").expect("manifest");
    fixture.run(&["build", "app.fpasprj"], 0);
    for target in ["app.fpasprj", "app.fpascp"] {
        let output = fixture.run(
            &[
                "debug",
                target,
                "--source-root",
                ".",
                "--break",
                "main.fpas:8",
                "--print",
                "First",
                "--print",
                "Second",
            ],
            0,
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("live debugger report");
        assert_eq!(report["outcome"], "breakpoint");
        let values = report["expressions"].as_array().expect("inspected values");
        for (value, owner, scalar) in [
            (&values[0]["value"], "InspectErrors.FileIssue", "settings"),
            (&values[1]["value"], "InspectErrors.TextIssue", "3"),
        ] {
            assert_eq!(value["status"], "available");
            assert_eq!(value["children"][0]["name"], "Cause");
            let cause = &value["children"][0]["value"];
            assert_eq!(cause["type"], owner);
            assert_eq!(cause["children"][0]["value"]["scalar"], scalar);
        }
        let dump_name = format!("{target}.failure.json");
        fixture.run(&["run", target, "--failure-dump", &dump_name], 2);
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.0.join(dump_name)).expect("failure capture"))
                .expect("failure JSON");
        assert_eq!(report["error"]["code"], "FP5010");
        assert_eq!(report["capture"]["incomplete"], false);
        for (name, owner) in [
            ("First", "InspectErrors.FileIssue"),
            ("Second", "InspectErrors.TextIssue"),
        ] {
            let binding = report["globals"]
                .as_array()
                .expect("captured globals")
                .iter()
                .find(|binding| binding["name"] == name)
                .expect("typed global");
            assert_eq!(binding["value"]["children"][0]["value"]["type"], owner);
        }
    }
}
