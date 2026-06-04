use super::*;
use crate::test_support::{write_library_fpasprj_with_exports, write_program_fpasprj_with_deps};

#[test]
fn library_exports_hide_units_from_dependents() {
    let dir = create_temp_dir("lib-exports-hide");
    let lib_dir = dir.join("libs/mylib");
    let app_dir = dir.join("apps/demo");
    let lib_project = lib_dir.join("mylib.fpasprj");
    let app_project = app_dir.join("demo.fpasprj");

    write_library_fpasprj_with_exports(&lib_project, &["src/**/*.fpas"], &["MyLib.Core"]);
    write_text(
        &lib_dir.join("src/core.fpas"),
        "unit MyLib.Core;\nuses MyLib.Internal;\nfunction Double(X: integer): integer;\nbegin\n  return Scale(X)\nend;\n",
    );
    write_text(
        &lib_dir.join("src/internal.fpas"),
        "unit MyLib.Internal;\nfunction Scale(X: integer): integer;\nbegin\n  return X + X\nend;\n",
    );

    let lib_dep = toml_path(&lib_project);
    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[lib_dep.as_str()],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses MyLib.Internal, Std.Console;\nbegin\n  WriteLn(Scale(3))\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    let error = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("non-exported unit must fail at link");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("not exported"),
        "expected export error, got: {error}"
    );
}

#[test]
fn library_exports_allow_public_unit_and_internal_uses() {
    let dir = create_temp_dir("lib-exports-ok");
    let lib_dir = dir.join("libs/mylib");
    let app_dir = dir.join("apps/demo");
    let lib_project = lib_dir.join("mylib.fpasprj");
    let app_project = app_dir.join("demo.fpasprj");

    write_library_fpasprj_with_exports(&lib_project, &["src/**/*.fpas"], &["MyLib.Core"]);
    write_text(
        &lib_dir.join("src/core.fpas"),
        "unit MyLib.Core;\nuses MyLib.Internal;\nfunction Double(X: integer): integer;\nbegin\n  return Scale(X)\nend;\n",
    );
    write_text(
        &lib_dir.join("src/internal.fpas"),
        "unit MyLib.Internal;\nfunction Scale(X: integer): integer;\nbegin\n  return X + X\nend;\n",
    );

    let lib_dep = toml_path(&lib_project);
    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[lib_dep.as_str()],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses MyLib.Core, Std.Console;\nbegin\n  WriteLn(Double(3))\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    let program = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect("program should link");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(program.name, "Demo");
}

#[test]
fn program_project_rejects_exports_section() {
    let dir = create_temp_dir("program-exports-forbidden");
    let project = dir.join("app.fpasprj");
    write_text(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[exports]
units = ["App.Core"]

[sources]
include = ["main.fpas"]
"#,
    );
    write_text(&dir.join("main.fpas"), "program App;\nbegin\nend.\n");

    let error = load_project_error(&project, "program exports must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("must not define `[exports]`"));
}
