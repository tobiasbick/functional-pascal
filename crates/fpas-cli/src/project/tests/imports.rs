use super::*;
use crate::test_support::{create_temp_dir, write_text};
use fpas_parser::Stmt;
use std::fs;

#[test]
fn build_program_rewrites_short_imports_to_qualified_names() {
    let dir = create_temp_dir("link-short-import");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &dir.join("src/main.fpas"),
        "program Main; uses App.Math; begin Add(1, 2) end.\n",
    );
    write_text(
        &dir.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );

    let loaded = load_project(&project_file).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link");
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
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
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
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
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

    let loaded = load_project(&project_file).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link");
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
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
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

    let loaded = load_project(&project_file).expect("project should load");
    let program = build_program(
        loaded.main.as_deref().expect("main path must exist"),
        &loaded.source_files,
    )
    .expect("project should link");
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
