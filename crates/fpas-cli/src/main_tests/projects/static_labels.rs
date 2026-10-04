//! Located rejection of computed scalar labels and imported static-label execution.

use super::*;

#[test]
fn scalar_labels_and_range_endpoints_report_located_non_constant_errors() {
    let cwd = create_temp_dir("static-case-labels");
    let source = cwd.join("main.fpas");
    for label in ["Lower", "Next()", "Lower..3", "1..Upper", "Next()..Next()"] {
        write_text(
            &source,
            &format!(
                "program Main;\nvar Lower: integer := 1;\nvar Upper: integer := 3;\nfunction Next(): integer; begin return 1; end function;\nbegin\ncase 2 of\nwhen {label}: null;\nelse null;\nend case;\nend program;\n"
            ),
        );
        let args = ["check", "--diagnostics", "json", &source.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "accepted {label}: {stderr}");
        assert!(stdout.is_empty());
        let diagnostics = stderr
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
            .collect::<Vec<_>>();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["code"] == "F2014"
                    && diagnostic["location"]["start"]["line"] == 7),
            "{label}: {stderr}"
        );
        assert!(!stderr.contains("F9001"), "{stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove temporary sources");
}

#[test]
fn imported_static_label_aliases_and_computed_guards_execute_after_sidecar_reuse() {
    let cwd = create_temp_dir("imported-static-case-labels");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public const Target: integer := 40 + 2; end unit;",
    );
    write_text(&cwd.join("main.fpas"), "program Main;
        uses Demo.Model as Model; uses Std.Console as Console;
        mutable var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return 42; end function;
        begin case 42 of when Model.Target: Console.WriteLn('static'); else null; end case;
          case 42 of when const Value if Value = Next(): Console.WriteLn('guard'); else null; end case;
          Console.WriteLn(Calls);
        end program;");
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "static\nguard\n1\n", "")
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}
