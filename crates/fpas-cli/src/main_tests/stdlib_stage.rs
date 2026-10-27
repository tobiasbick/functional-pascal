use super::*;

fn stage_args(extra: &[&str]) -> Vec<String> {
    [
        "stdlib",
        "stage",
        "--source",
        "staging",
        "--output",
        "distribution",
    ]
    .into_iter()
    .chain(extra.iter().copied())
    .map(String::from)
    .collect()
}

#[test]
fn stdlib_stage_previews_then_compiles_and_publishes() {
    let cwd = create_temp_dir("stdlib-stage");
    write_text(
        &cwd.join("staging/stdlib.fpasprj"),
        r#"[project]
name = "distribution-fixture"
kind = "library"
[exports]
units = ["Std.DistributionFixture"]
[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("staging/Std/DistributionFixture.fpas"),
        "unit Std.DistributionFixture; const Answer: integer := 42; end unit;",
    );

    let (code, stdout, stderr) =
        support::run_cli_args_and_capture_output(&stage_args(&["--dry-run"]), &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("Would recompile"));
    assert!(!cwd.join("distribution").exists());
    assert!(!cwd.join("staging/Std/DistributionFixture.fpascu").exists());

    let (code, stdout, stderr) = support::run_cli_args_and_capture_output(&stage_args(&[]), &cwd);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("Compiled 1 standard-library unit(s), reused 0."));
    assert!(
        cwd.join("distribution/Std/DistributionFixture.fpascu")
            .is_file()
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(&stage_args(&[]), &cwd);
    assert_eq!(code, 0, "repeated staging failed: {stderr}");
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn stdlib_stage_reports_invalid_arguments_and_staging_failure() {
    let cwd = create_temp_dir("stdlib-stage-errors");
    for args in [
        vec!["stdlib"],
        vec!["stdlib", "stage", "--source", "staging"],
        vec!["stdlib", "stage", "--source", "--output", "distribution"],
        vec![
            "stdlib",
            "stage",
            "--source",
            "staging",
            "--source",
            "other",
            "--output",
            "distribution",
        ],
        vec![
            "stdlib",
            "stage",
            "--source",
            "staging",
            "--output",
            "distribution",
            "--unknown",
        ],
    ] {
        let args = args.into_iter().map(String::from).collect::<Vec<_>>();
        let (code, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_ne!(code, 0, "{args:?}");
        assert!(stdout.is_empty());
        assert!(stderr.contains("fpas stdlib stage --help"), "{stderr}");
    }
    let (code, stdout, stderr) = support::run_cli_args_and_capture_output(&stage_args(&[]), &cwd);
    assert_ne!(code, 0);
    assert!(stdout.is_empty());
    assert!(!stderr.is_empty());
    assert!(!cwd.join("distribution").exists());
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}
