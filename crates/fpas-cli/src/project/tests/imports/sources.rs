use super::{Decl, build_program, load_project, toml_path, write_program_project_file};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

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
        &loaded.link_meta,
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
