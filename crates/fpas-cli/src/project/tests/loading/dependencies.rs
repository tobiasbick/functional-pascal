use super::*;
use crate::test_support::{
    write_library_fpasprj, write_library_fpasprj_with_deps, write_program_fpasprj,
    write_program_fpasprj_with_deps,
};

#[test]
fn program_project_loads_relative_library_dependency() {
    let dir = create_temp_dir("dep-relative-lib");
    let lib_dir = dir.join("libs").join("acme");
    let app_dir = dir.join("apps").join("demo");
    let lib_project = lib_dir.join("acme.fpasprj");
    let app_project = app_dir.join("demo.fpasprj");

    write_library_fpasprj(&lib_project, &["src/**/*.fpas"]);
    write_text(
        &lib_dir.join("src/math.fpas"),
        "unit Acme.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../../libs/acme/acme.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses Acme.Math, Std.Console;\nbegin\n  WriteLn(Add(2, 5))\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    assert_eq!(loaded.source_files.len(), 1);
    let program = build_program(
        loaded.main.as_deref().expect("main path"),
        &loaded.source_files,
    )
    .expect("program should link");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(program.name, "Demo");
}

#[test]
fn program_project_loads_absolute_library_dependency() {
    let dir = create_temp_dir("dep-absolute-lib");
    let lib_dir = dir.join("vendor-lib");
    let app_dir = dir.join("consumer");
    let lib_project = lib_dir.join("vendor.fpasprj");
    let app_project = app_dir.join("consumer.fpasprj");
    let absolute_lib = toml_path(&lib_project);

    write_library_fpasprj(&lib_project, &["src/**/*.fpas"]);
    write_text(
        &lib_dir.join("src/greet.fpas"),
        "unit Vendor.Greet;\nconst Message: string := 'from vendor';\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[&absolute_lib],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Consumer;\nuses Vendor.Greet, Std.Console;\nbegin\n  WriteLn(Message)\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
}

#[test]
fn library_project_loads_transitive_dependencies() {
    let dir = create_temp_dir("dep-transitive");
    let base_dir = dir.join("libs/base");
    let util_dir = dir.join("libs/util");
    let base_project = base_dir.join("base.fpasprj");
    let util_project = util_dir.join("util.fpasprj");

    write_library_fpasprj(&base_project, &["src/**/*.fpas"]);
    write_text(
        &base_dir.join("src/base.fpas"),
        "unit Lib.Base;\nconst Tag: string := 'base';\n",
    );

    write_library_fpasprj_with_deps(&util_project, &["src/**/*.fpas"], &["../base/base.fpasprj"]);
    write_text(
        &util_dir.join("src/util.fpas"),
        "unit Lib.Util;\nuses Lib.Base;\nfunction Label(): string;\nbegin\n  return Tag\nend;\n",
    );

    let loaded = load_project_ok(&util_project);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 2);
}

#[test]
fn cyclic_project_dependencies_are_rejected() {
    let dir = create_temp_dir("dep-cycle");
    let left_dir = dir.join("left");
    let right_dir = dir.join("right");
    let left_project = left_dir.join("left.fpasprj");
    let right_project = right_dir.join("right.fpasprj");

    write_library_fpasprj_with_deps(
        &left_project,
        &["src/**/*.fpas"],
        &["../right/right.fpasprj"],
    );
    write_text(&left_dir.join("src/left.fpas"), "unit Cycle.Left;\n");

    write_library_fpasprj_with_deps(
        &right_project,
        &["src/**/*.fpas"],
        &["../left/left.fpasprj"],
    );
    write_text(&right_dir.join("src/right.fpas"), "unit Cycle.Right;\n");

    let error = load_project_error(&left_project, "cyclic project dependencies must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Cyclic project dependency detected"),
        "expected cycle error, got: {error}"
    );
}

#[test]
fn program_project_dependency_must_be_library() {
    let dir = create_temp_dir("dep-program-forbidden");
    let helper_dir = dir.join("helper");
    let app_dir = dir.join("app");
    let helper_project = helper_dir.join("helper.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_program_fpasprj(&helper_project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &helper_dir.join("src/main.fpas"),
        "program Helper;\nbegin\nend.\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../helper/helper.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program App;\nbegin\nend.\n",
    );

    let error = load_project_error(&app_project, "program dependency must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("must be a library project"),
        "expected library-only dependency error, got: {error}"
    );
}

#[test]
fn duplicate_unit_across_dependency_projects_is_rejected() {
    let dir = create_temp_dir("dep-duplicate-unit");
    let lib_a_dir = dir.join("lib-a");
    let lib_b_dir = dir.join("lib-b");
    let app_dir = dir.join("app");
    let lib_a_project = lib_a_dir.join("a.fpasprj");
    let lib_b_project = lib_b_dir.join("b.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_library_fpasprj(&lib_a_project, &["src/**/*.fpas"]);
    write_text(&lib_a_dir.join("src/shared.fpas"), "unit Shared.Name;\n");

    write_library_fpasprj(&lib_b_project, &["src/**/*.fpas"]);
    write_text(&lib_b_dir.join("src/shared.fpas"), "unit Shared.Name;\n");

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../lib-a/a.fpasprj", "../lib-b/b.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program App;\nbegin\nend.\n",
    );

    let error = load_project_error(&app_project, "duplicate units across deps must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Duplicate unit name `Shared.Name`"),
        "expected duplicate unit error, got: {error}"
    );
}

#[test]
fn missing_project_dependency_path_is_rejected() {
    let dir = create_temp_dir("dep-missing-path");
    let app_project = dir.join("app.fpasprj");

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../missing/lib.fpasprj"],
    );
    write_text(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program App;\nbegin\nend.\n",
    );

    let error = load_project_error(&app_project, "missing dependency path must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("dependencies.projects") && error.contains("does not exist"),
        "expected missing dependency error, got: {error}"
    );
}
