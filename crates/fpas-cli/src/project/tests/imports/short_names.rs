use super::{
    Decl, DesignatorPart, Stmt, build_program, load_and_build_program, load_project,
    write_program_project_file,
};
use crate::test_support::{create_temp_dir, write_text};
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
        &loaded.link_meta,
    )
    .expect_err("ambiguous short name should fail at use site");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(error.contains("Ambiguous imported symbol `Add`"));
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
