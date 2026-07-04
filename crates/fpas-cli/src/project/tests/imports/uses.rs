use super::{load_and_build_program, write_program_project_file};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;

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
