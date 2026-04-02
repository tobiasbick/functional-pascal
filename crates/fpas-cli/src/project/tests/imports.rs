use super::support::load_and_build_program;
use super::support::toml_path;
use super::support::write_program_project_file;
use super::*;
use crate::test_support::{create_temp_dir, write_text};
use fpas_parser::Stmt;
use std::fs;

#[test]
fn build_program_rewrites_short_imports_to_qualified_names() {
    let dir = create_temp_dir("link-short-import");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Math; begin Add(1, 2) end.\n",
    );
    write_text(
        &dir.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Decl::Function(f) if f.name == "App.Math.Add"))
    );
    assert!(matches!(
        &program.body[0],
        Stmt::Call { designator, .. }
            if matches!(
                designator.parts.as_slice(),
                [
                    DesignatorPart::Ident(a, _),
                    DesignatorPart::Ident(b, _),
                    DesignatorPart::Ident(c, _)
                ] if a == "App" && b == "Math" && c == "Add"
            )
    ));
}

#[test]
fn build_program_reports_ambiguous_import_at_use_site() {
    let dir = create_temp_dir("link-ambiguous");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Math, App.Advanced; begin Add(1, 2) end.\n",
    );
    write_text(
        &dir.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );
    write_text(
        &dir.join("src/advanced.fpas"),
        "unit App.Advanced;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A - B\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let error = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect_err("ambiguous short name should fail at use site");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Ambiguous imported symbol `Add`"));
}

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
fn build_program_keeps_private_unit_symbols_internal() {
    let dir = create_temp_dir("link-private-internal");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Lib; begin PublicValue() end.\n",
    );
    write_text(
        &dir.join("src/lib.fpas"),
        "\
unit App.Lib;

private function SecretValue(): integer;
begin
  return 10
end;

function PublicValue(): integer;
begin
  return SecretValue()
end;
",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(program.declarations.iter().any(
        |decl| matches!(decl, Decl::Function(f) if f.name == "App.Lib.__private__.SecretValue")
    ));
    assert!(
        program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Decl::Function(f) if f.name == "App.Lib.PublicValue"))
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
fn build_program_preserves_stable_deduplicated_std_uses_from_program_and_units() {
    let dir = create_temp_dir("link-std-uses");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses Std.Console, App.Beta, App.Alpha; begin end.\n",
    );
    write_text(
        &dir.join("src/alpha.fpas"),
        "unit App.Alpha;\nuses Std.Console, Std.Math;\nfunction Alpha(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &dir.join("src/beta.fpas"),
        "unit App.Beta;\nuses Std.Math, Std.Array;\nfunction Beta(): integer;\nbegin\n  return 2\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    let uses = program
        .uses
        .iter()
        .map(|used| used.parts.join("."))
        .collect::<Vec<_>>();

    assert_eq!(uses, vec!["Std.Console", "Std.Math", "Std.Array"]);
}

#[test]
fn build_program_does_not_treat_private_collision_as_ambiguous_import() {
    let dir = create_temp_dir("link-private-not-ambiguous");
    let project_file = dir.join("app.fpasprj");
    write_program_project_file(&project_file, &["src/*.fpas"]);
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Math, App.Advanced; begin Add(1, 2) end.\n",
    );
    write_text(
        &dir.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );
    write_text(
        &dir.join("src/advanced.fpas"),
        "unit App.Advanced;\nprivate function Add(A: integer; B: integer): integer;\nbegin\n  return A - B\nend;\n\nfunction UseSecret(): integer;\nbegin\n  return Add(4, 1)\nend;\n",
    );

    let program = load_and_build_program(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(matches!(
        &program.body[0],
        Stmt::Call { designator, .. }
            if matches!(
                designator.parts.as_slice(),
                [
                    DesignatorPart::Ident(a, _),
                    DesignatorPart::Ident(b, _),
                    DesignatorPart::Ident(c, _)
                ] if a == "App" && b == "Math" && c == "Add"
            )
    ));
}

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
    )
    .expect_err("unknown unit must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Available units: App.Alpha, App.Beta."));
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

#[test]
fn build_program_accepts_relative_and_absolute_main_entries_in_same_project() {
    let dir = create_temp_dir("link-relative-absolute-main-entry");
    let main_path = dir.join("src/main.fpas");
    let main_path_text = toml_path(&main_path);
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/main.fpas", "{main_path_text}", "src/lib.fpas"]
"#
        ),
    );
    write_text(&main_path, "program Main; uses App.Lib; begin Lib() end.\n");
    write_text(
        &dir.join("src/lib.fpas"),
        "unit App.Lib;\nfunction Lib(): integer;\nbegin\n  return 1\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
    assert!(
        program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Decl::Function(function) if function.name == "App.Lib.Lib"))
    );
}
