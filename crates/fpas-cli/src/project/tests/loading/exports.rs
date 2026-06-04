use super::*;
use crate::test_support::{
    write_library_fpasprj_with_deps, write_library_fpasprj_with_exports,
    write_program_fpasprj_with_deps,
};

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
fn transitive_library_dependency_respects_export_list() {
    let dir = create_temp_dir("lib-exports-transitive");
    let base_dir = dir.join("libs/base");
    let util_dir = dir.join("libs/util");
    let app_dir = dir.join("apps/demo");
    let base_project = base_dir.join("base.fpasprj");
    let util_project = util_dir.join("util.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_library_fpasprj_with_exports(&base_project, &["src/**/*.fpas"], &["Lib.Base"]);
    write_text(
        &base_dir.join("src/base.fpas"),
        "unit Lib.Base;\nfunction Tag(): string;\nbegin\n  return 'ok'\nend;\n",
    );
    write_text(
        &base_dir.join("src/internal.fpas"),
        "unit Lib.Base.Internal;\nfunction Hidden(): string;\nbegin\n  return 'secret'\nend;\n",
    );

    write_library_fpasprj_with_deps(&util_project, &["src/**/*.fpas"], &["../base/base.fpasprj"]);
    write_text(
        &util_dir.join("src/util.fpas"),
        "unit Lib.Util;\nuses Lib.Base;\nfunction Label(): string;\nbegin\n  return Tag()\nend;\n",
    );

    let util_dep = toml_path(&util_project);
    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[util_dep.as_str()],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses Lib.Base.Internal, Std.Console;\nbegin\n  WriteLn(Hidden())\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    let error = build_program(
        loaded.main.as_deref().expect("main"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("transitive non-exported unit must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("not exported"),
        "expected export error, got: {error}"
    );
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
