use super::{Decl, load_and_build_program, write_program_project_file};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

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
