use super::*;

fn arguments<'a>(extra: &'a [&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "add-parameter",
        "--symbol",
        "Demo.Math.Answer",
        "--name",
        "Extra",
        "--type",
        "integer",
        "--mode",
        "value",
        "--argument",
        "7",
        "--context",
        "suite.fpasworkspace",
        "--std-lib",
        "std",
    ];
    args.extend_from_slice(extra);
    args
}

#[test]
fn parameter_preview_matches_core_and_apply_is_exact_stale_safe_and_repeatable() {
    use fpas_language_service::{
        CancellationToken, ParameterMode, RefactorRecipe, SourceSelector, SourceSession,
    };
    let f = Fixture::new();
    let (preview, bytes) = f.run(&arguments(&["--out", "parameter.json"]), 0);
    assert_eq!(fs::read(f.0.join("parameter.json")).unwrap(), bytes);
    assert_eq!(preview["result"]["status"], "prepared");
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
    assert_eq!(fs::read_to_string(f.0.join("app/main.fpas")).unwrap(), MAIN);
    let cancel = CancellationToken::new();
    let mut session = SourceSession::load(
        &f.0.join("suite.fpasworkspace"),
        Some(&f.0.join("std")),
        &[],
        &cancel,
    )
    .unwrap();
    let prepared = session
        .prepare_refactor(
            &RefactorRecipe::AddParameter {
                selector: SourceSelector::Symbol {
                    name: "Demo.Math.Answer".into(),
                },
                name: "Extra".into(),
                r#type: "integer".into(),
                mode: ParameterMode::Value,
                argument: "7".into(),
            },
            &cancel,
        )
        .unwrap();
    assert_eq!(
        preview["result"]["plan"],
        serde_json::to_value(prepared.plan).unwrap()
    );
    f.write("lib/unrelated.fpas", "unit Demo.Unrelated; end unit;");
    let stale = f.run(&["apply", "--plan", "parameter.json"], 1).0;
    assert_eq!(stale["diagnostics"][0]["code"], "stale_plan");
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
    fs::remove_file(f.0.join("lib/unrelated.fpas")).unwrap();
    assert_eq!(
        f.run(&["apply", "--plan", "parameter.json"], 0).0["result"]["status"],
        "applied"
    );
    assert_eq!(
        fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(),
        UNIT.replace("Answer()", "Answer(Extra: integer)")
    );
    assert_eq!(
        fs::read_to_string(f.0.join("app/main.fpas")).unwrap(),
        MAIN.replace("M.answer()", "M.answer(7)")
            .replace("M.ANSWER()", "M.ANSWER(7)")
    );
    assert_eq!(
        f.run(&["apply", "--plan", "parameter.json"], 0).0["result"]["status"],
        "already_applied"
    );
}

#[test]
fn parameter_blockers_and_incomplete_options_never_publish_subset_edits() {
    let f = Fixture::new();
    let source = MAIN.replace(
        "  discard '😀 Answer';",
        "  const Callback := M.Answer; discard '😀 Answer';",
    );
    f.write("app/main.fpas", &source);
    let refused = f.run(&arguments(&["--out", "refused.json"]), 1).0;
    assert!(refused["result"]["plan"].is_null());
    assert!(
        refused["result"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["code"] == "callable_value" && b["start"].is_number())
    );
    assert!(!f.0.join("refused.json").exists());
    for option in ["--name", "--type", "--mode", "--argument"] {
        let mut args = arguments(&[]);
        let at = args.iter().position(|s| *s == option).unwrap();
        args.drain(at..at + 2);
        assert_eq!(
            f.run(&args, 1).0["diagnostics"][0]["code"],
            "invalid_request"
        );
    }
    assert_eq!(
        fs::read_to_string(f.0.join("app/main.fpas")).unwrap(),
        source
    );
    assert_eq!(fs::read_to_string(f.0.join("lib/math.fpas")).unwrap(), UNIT);
}
