//! Generic constraint forwarding through compiled-unit interfaces and real CLI diagnostics.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use super::*;

const NUMBERS: &str = "unit Demo.Numbers;\npublic function Twice of (T: Numeric)(Value: T): T;\nbegin return Value + Value; end function;\nend unit;";

#[test]
fn imported_generic_constraint_forwarding_runs_through_the_cli() {
    let cwd = create_temp_dir("generic-constraint-forwarding");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("numbers.fpas"), NUMBERS);
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Numbers as Numbers; uses Std.Console as Console;\n         function Forward of (U: Numeric)(Value: U): U;\n         begin return Numbers.Twice(Value); end function;\n         begin Console.WriteLn(Forward(21)); Console.WriteLn(Forward(1.25)); end program;",
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (0, "42\n2.5\n", "")
    );
}

#[test]
fn imported_generic_constraint_bypass_has_a_located_json_diagnostic() {
    for (name, constraint) in [("T", ""), ("U", ": Comparable")] {
        let cwd = create_temp_dir("generic-constraint-bypass");
        let project = cwd.join("app.fpasprj");
        support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
        write_text(&cwd.join("numbers.fpas"), NUMBERS);
        write_text(
            &cwd.join("main.fpas"),
            &format!(
                "program Main;\nuses Demo.Numbers as Numbers;\nfunction Forward of ({name}{constraint})(Value: {name}): {name};\nbegin return Numbers.Twice(Value); end function;\nbegin discard Forward('x'); end program;\n"
            ),
        );
        let args =
            ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        fs::remove_dir_all(&cwd).expect("remove temporary project");
        assert_eq!(exit, 1, "{stderr}");
        assert!(stdout.is_empty());
        let records = stderr
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1, "{stderr}");
        assert_eq!(records[0]["code"], "F2013");
        assert_eq!(records[0]["location"]["start"]["line"], 4);
        assert!(
            records[0]["message"]
                .as_str()
                .is_some_and(|message| message.contains("Numeric"))
        );
    }
}
