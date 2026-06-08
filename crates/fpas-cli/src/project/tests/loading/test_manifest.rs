use super::*;

#[test]
fn test_project_accepts_overrides_section() {
    let dir = create_temp_dir("test-manifest-overrides");
    let project_file = dir.join("tests.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "tests"
kind = "test"

[sources]
include = ["*.fpas"]

[test.overrides."alpha_test.fpas"]
script = "alpha.script.toml"
headless_graph = true
"#,
    );
    write_text(
        &dir.join("alpha_test.fpas"),
        "program A;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );
    write_text(
        &dir.join("alpha.script.toml"),
        "[[event]]\ntype = \"readln\"\nline = \"x\"\n",
    );

    let loaded = load_project_ok(&project_file);
    let override_cfg = loaded
        .test_manifest
        .override_for(&dir.join("alpha_test.fpas"))
        .expect("override must exist");
    assert_eq!(override_cfg.script, Some(dir.join("alpha.script.toml")));
    assert_eq!(override_cfg.headless_graph, Some(true));

    fs::remove_dir_all(&dir).expect("temp directory must be removed");
}

#[test]
fn program_project_rejects_test_section() {
    let dir = create_temp_dir("program-test-section");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[sources]
include = ["*.fpas"]

[test.overrides."demo_test.fpas"]
script = "demo.script.toml"
"#,
    );
    write_text(&dir.join("main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "program must reject [test]");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("must not define `[test]`"));
}
