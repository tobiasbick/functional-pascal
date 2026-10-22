use super::*;

const STACK: &str = "program Stack;\nvar Total: integer := 1;\nfunction Add(X: integer): integer;\nbegin\n  const Adjusted := X + 1;\n  Total := Adjusted;\n  return Adjusted;\nend function;\nbegin\n  Total := Add(40);\nend.\n";

#[test]
fn first_breakpoint_stops_before_statement_and_keeps_lexical_stack() {
    let f = Fixture::new(STACK);
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--break",
            "main.fpas:6",
            "--break",
            "main.fpas:7",
            "--print",
            "Adjusted",
            "--print",
            "Total",
        ],
        0,
    );
    assert_eq!(r["outcome"], "breakpoint");
    assert_eq!(r["stop"]["location"]["line"], 6);
    assert_eq!(r["expressions"][0]["value"]["scalar"], "41");
    assert_eq!(r["expressions"][1]["value"]["scalar"], "1");
    let frames = r["tasks"][0]["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 2);
    assert!(
        frames[0]["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|binding| binding["name"] == "X" && binding["kind"] == "parameter")
    );
    let caller = f.run(
        &[
            "debug",
            "main.fpas",
            "--break",
            "main.fpas:6",
            "--frame",
            "1",
            "--print",
            "Total",
        ],
        0,
    );
    assert_eq!(caller["expressions"][0]["frame"], 1);
}
#[test]
fn expression_failures_are_independent_and_context_never_falls_back() {
    let f = Fixture::new(STACK);
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--print",
            "Total",
            "--print",
            "Adjusted",
        ],
        3,
    );
    assert_eq!(r["outcome"], "completed");
    assert_eq!(r["expressions"][0]["value"]["scalar"], "41");
    assert!(!r["expressions"][1]["error"].is_null());
    for args in [vec!["--task", "999"], vec!["--frame", "999"]] {
        let mut request = vec!["debug", "main.fpas", "--break", "main.fpas:6"];
        request.extend(args);
        let r = f.run(&request, 3);
        assert!(
            r["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "inspection_context")
        );
    }
    f.run(&["debug", "main.fpas", "--frame", "0"], 3);
}
#[test]
fn runtime_failure_preserves_diagnostic_and_wins_over_bad_print() {
    let f = Fixture::new("program Failure; var Total: integer := 42; begin panic('broken'); end.");
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--print",
            "Total",
            "--print",
            "Unknown",
        ],
        2,
    );
    assert_eq!(r["outcome"], "runtime_error");
    assert_eq!(r["expressions"][0]["value"]["scalar"], "42");
    assert!(!r["diagnostics"][0]["code"].is_null());
    assert!(!r["tasks"][0]["frames"].as_array().unwrap().is_empty());
}
#[test]
fn worker_stop_exposes_all_stop_tasks() {
    let f = Fixture::new(
        "program Worker;\nuses Std.Tasks as Tasks;\nfunction Work(Value: integer): integer;\nbegin\n const Adjusted := Value + 1;\n return Adjusted;\nend function;\nbegin\n scope\n  const Child := go Work(40);\n  discard Tasks.Wait(Child);\n end scope;\nend.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--break",
            "main.fpas:6",
            "--print",
            "Adjusted",
        ],
        0,
    );
    assert_ne!(r["stop"]["task_id"], 0);
    assert_eq!(r["expressions"][0]["value"]["scalar"], "41");
    assert!(
        r["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|task| task["id"] == 0)
    );
    let peer = f.run(
        &[
            "debug",
            "main.fpas",
            "--break",
            "main.fpas:6",
            "--task",
            "0",
            "--print",
            "Child",
        ],
        0,
    );
    assert_eq!(peer["expressions"][0]["task_id"], 0);
    assert_eq!(peer["expressions"][0]["value"]["kind"], "task");
    let main = peer["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["id"] == 0)
        .unwrap();
    assert_eq!(main["state"], "waiting");
    assert_eq!(main["frames"][0]["location"]["line"], 11);
}
#[test]
fn value_bounds_are_explicit_without_failing_successful_inspection() {
    let f = Fixture::new(
        "program Bounds; var Numbers: array of integer := []; var Text: string := ''; begin for I: integer := 0 to 100 do Numbers.PushInPlace(I); end for; Text := 'é'.RepeatStr(5000); end.",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--print",
            "Numbers",
            "--print",
            "Text",
        ],
        0,
    );
    assert_eq!(
        r["expressions"][0]["value"]["children"]
            .as_array()
            .unwrap()
            .len(),
        64
    );
    assert_eq!(r["expressions"][0]["value"]["reason"], "child_limit");
    assert_eq!(
        r["expressions"][1]["value"]["scalar"]
            .as_str()
            .unwrap()
            .chars()
            .count(),
        4096
    );
    assert_eq!(r["capture"]["incomplete"], true);
}

#[test]
fn unresolved_dotted_names_never_construct_enum_values() {
    let f = Fixture::new("program Defaults; type Choice = enum Item; end enum; begin null; end.");
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--print",
            "Choice.Item",
            "--print",
            "1 + 2",
        ],
        3,
    );
    assert!(!r["expressions"][0]["error"].is_null());
    assert_eq!(r["expressions"][1]["value"]["scalar"], "3");
}

#[test]
fn runtime_failure_frames_use_the_faulting_instruction_location() {
    let f = Fixture::new(
        "program Fault;\nprocedure Boom(N: integer);\nbegin\n const Before := N + 1;\n panic('bad');\n const After := N + 2;\nend procedure;\nbegin\n Boom(41);\nend.\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--print",
            "Before",
            "--print",
            "After",
        ],
        2,
    );
    assert_eq!(r["stop"]["location"]["line"], 5);
    let top = &r["tasks"][0]["frames"][0];
    assert_eq!(top["location"]["line"], 5);
    assert!(top["reason"].is_null());
    assert_eq!(r["expressions"][0]["value"]["scalar"], "42");
    assert!(!r["expressions"][1]["error"].is_null());
}

#[test]
fn runtime_callback_frame_and_expression_context_describe_the_same_function() {
    let f = Fixture::new(
        "program Callback;\nfunction FailOnSecond(C: string): string;\nbegin\n if C = 'b' then\n  panic('callback');\n end if;\n return C;\nend function;\nbegin\n const Mapped := 'abc'.Map(FailOnSecond);\nend.\n",
    );
    let r = f.run(&["debug", "main.fpas", "--print", "C"], 2);
    assert_eq!(r["expressions"][0]["value"]["scalar"], "b");
    assert_eq!(r["stop"]["location"]["line"], 5);
    assert_eq!(r["tasks"][0]["frames"][0]["location"]["line"], 5);
    assert!(
        r["tasks"][0]["frames"][0]["function"]
            .as_str()
            .unwrap()
            .contains("failonsecond")
    );
    assert!(r["tasks"][0]["frames"].as_array().unwrap().len() >= 2);
    let caller = f.run(&["debug", "main.fpas", "--frame", "1", "--print", "C"], 2);
    assert!(!caller["expressions"][0]["error"].is_null());
}
