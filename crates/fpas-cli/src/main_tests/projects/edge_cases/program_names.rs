//! Source, project, and artifact paths accept declarations matching the program heading.
//!
//! **Documentation:** `docs/pascal/getting-started/first-program.md`

use super::*;

fn assert_program_name_collision(name: &str, declarations: &str, body: &str) {
    let std_lib = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root")
        .join("lib");
    for heading in [name.to_string(), name.to_ascii_uppercase()] {
        let cwd = create_temp_dir("program-name-collision");
        let source = cwd.join("src/main.fpas");
        let project = cwd.join("app.fpasprj");
        support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
        write_text(
            &source,
            &format!("program {heading};\n{declarations}\nbegin\n{body}\nend.\n"),
        );
        for input in [&source, &project] {
            for command in ["check", "run"] {
                let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
                    &[
                        command.into(),
                        "--std-lib".into(),
                        std_lib.to_string_lossy().into_owned(),
                        input.to_string_lossy().into_owned(),
                    ],
                    &cwd,
                );
                assert_eq!(exit, 0, "{heading} {command} {input:?}: {stderr}");
                assert!(stdout.is_empty(), "{stdout}");
            }
        }
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &[
                "build".into(),
                "--std-lib".into(),
                std_lib.to_string_lossy().into_owned(),
                project.to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        assert_eq!(exit, 0, "{heading} build: {stderr}");
        let artifact = cwd.join("app.fpascp");
        assert!(artifact.is_file(), "compiled program artifact");
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), artifact.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(exit, 0, "{heading} artifact run: {stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        fs::remove_dir_all(&cwd).expect("remove fixture");
    }
}

#[test]
fn program_heading_matching_function_compiles_on_all_cli_paths() {
    assert_program_name_collision(
        "Demo",
        "function Demo(N: integer): integer; begin
           if N = 0 then return 1; end if;
           return N * Demo(N - 1); end function;",
        "if Demo(4) <> 24 then panic('recursive call'); end if;",
    );
}

#[test]
fn program_heading_matching_procedure_compiles_on_all_cli_paths() {
    assert_program_name_collision(
        "Demo",
        "var Called: boolean := false;
         procedure Demo(); begin Called := true; end procedure;",
        "Demo(); if not Called then panic('procedure call'); end if;",
    );
}

#[test]
fn program_heading_matching_global_compiles_on_all_cli_paths() {
    assert_program_name_collision(
        "Counter",
        "var Counter: integer := 40 + 1;",
        "Counter := Counter + 1;
         if Counter <> 42 then panic('global initializer'); end if;",
    );
}

#[test]
fn program_heading_matching_record_compiles_on_all_cli_paths() {
    assert_program_name_collision(
        "Point",
        "type Point = record X: integer := 7; end record;",
        "const P: Point := Point();
         if P.X <> 7 then panic('record default'); end if;",
    );
}

#[test]
fn program_heading_matching_enum_compiles_on_all_cli_paths() {
    assert_program_name_collision(
        "Choice",
        "type Choice = enum Number(Value: integer); Empty; end enum;",
        "const Selected: Choice := Choice.Number(Value := 7);
         if Selected <> Choice.Number(Value := 7) then panic('enum value'); end if;",
    );
}
