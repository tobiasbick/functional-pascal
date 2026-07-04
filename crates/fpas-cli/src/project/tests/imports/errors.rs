use super::{build_program, load_and_build_program, load_project, write_program_project_file};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

#[test]
fn build_program_rejects_duplicate_top_level_names_inside_one_unit() {
    let dir = create_temp_dir("link-duplicate-top-level");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Math; begin end.\n",
    );
    write_text(
        &dir.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(): integer;\nbegin\n  return 1\nend;\n\nprocedure Add();\nbegin\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("duplicate names in one unit must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Duplicate declaration `Add` in unit `App.Math`"));
}

#[test]
fn build_program_reports_unknown_unit_with_sorted_available_units() {
    let dir = create_temp_dir("link-unknown-unit-sorted");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Missing; begin end.\n",
    );
    write_text(
        &dir.join("src/beta.fpas"),
        "unit App.Beta;\nfunction Beta(): integer;\nbegin\n  return 2\nend;\n",
    );
    write_text(
        &dir.join("src/alpha.fpas"),
        "unit App.Alpha;\nfunction Alpha(): integer;\nbegin\n  return 1\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("unknown unit must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Available units: App.Alpha, App.Beta."));
}
