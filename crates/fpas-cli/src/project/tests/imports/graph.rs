use super::{
    Decl, build_program, load_and_build_program, load_project, write_program_project_file,
};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

#[test]
fn build_program_includes_transitive_unit_dependencies() {
    let dir = create_temp_dir("link-transitive");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Top; begin Top() end.\n",
    );
    write_text(
        &dir.join("src/top.fpas"),
        "unit App.Top;\nuses App.Core;\nfunction Top(): integer;\nbegin\n  return Core()\nend;\n",
    );
    write_text(
        &dir.join("src/core.fpas"),
        "unit App.Core;\nfunction Core(): integer;\nbegin\n  return 1\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Decl::Function(f) if f.name == "App.Core.Core"))
    );
    assert!(
        program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Decl::Function(f) if f.name == "App.Top.Top"))
    );
}
#[test]
fn build_program_links_independent_units_in_stable_order() {
    let dir = create_temp_dir("link-stable-order");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Beta, App.Alpha; begin end.\n",
    );
    write_text(
        &dir.join("src/alpha.fpas"),
        "unit App.Alpha;\nfunction Alpha(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &dir.join("src/beta.fpas"),
        "unit App.Beta;\nfunction Beta(): integer;\nbegin\n  return 2\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    let function_names = program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            Decl::Function(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(function_names, vec!["App.Alpha.Alpha", "App.Beta.Beta"]);
}
#[test]
fn build_program_ignores_broken_imports_in_unreachable_units() {
    let dir = create_temp_dir("link-unreachable-broken-import");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Live; begin Live() end.\n",
    );
    write_text(
        &dir.join("src/live.fpas"),
        "unit App.Live;\nfunction Live(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &dir.join("src/dead.fpas"),
        "unit App.Dead;\nuses App.Missing;\nfunction Dead(): integer;\nbegin\n  return 2\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        program.declarations.iter().any(
            |decl| matches!(decl, Decl::Function(function) if function.name == "App.Live.Live")
        )
    );
    assert!(
        !program.declarations.iter().any(
            |decl| matches!(decl, Decl::Function(function) if function.name == "App.Dead.Dead")
        )
    );
}
#[test]
fn build_program_reports_three_unit_cycle_with_stable_path() {
    let dir = create_temp_dir("link-stable-cycle-path");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.A; begin end.\n",
    );
    write_text(
        &dir.join("src/a.fpas"),
        "unit App.A;\nuses App.B;\nfunction A(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &dir.join("src/b.fpas"),
        "unit App.B;\nuses App.C;\nfunction B(): integer;\nbegin\n  return 2\nend;\n",
    );
    write_text(
        &dir.join("src/c.fpas"),
        "unit App.C;\nuses App.A;\nfunction C(): integer;\nbegin\n  return 3\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
        &loaded.link_meta,
    )
    .expect_err("cycle must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Cyclic unit dependency detected: App.A -> App.B -> App.C -> App.A"));
}
#[test]
fn build_program_links_independent_subgraphs_in_stable_dependency_order() {
    let dir = create_temp_dir("link-stable-subgraphs");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Beta, App.Alpha; begin end.\n",
    );
    write_text(
        &dir.join("src/alpha.fpas"),
        "unit App.Alpha;\nuses App.Alpha.Core;\nfunction Alpha(): integer;\nbegin\n  return AlphaCore()\nend;\n",
    );
    write_text(
        &dir.join("src/alpha_core.fpas"),
        "unit App.Alpha.Core;\nfunction AlphaCore(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &dir.join("src/beta.fpas"),
        "unit App.Beta;\nuses App.Beta.Core;\nfunction Beta(): integer;\nbegin\n  return BetaCore()\nend;\n",
    );
    write_text(
        &dir.join("src/beta_core.fpas"),
        "unit App.Beta.Core;\nfunction BetaCore(): integer;\nbegin\n  return 2\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    let function_names = program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            Decl::Function(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        function_names,
        vec![
            "App.Alpha.Core.AlphaCore",
            "App.Alpha.Alpha",
            "App.Beta.Core.BetaCore",
            "App.Beta.Beta",
        ]
    );
}
