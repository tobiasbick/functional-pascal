use super::*;

#[test]
fn operator_and_bit_errors_use_the_real_cli_json_stream() {
    let root = temp_dir();
    for (command, expression, message) in [
        ("check", "true and false or true", "Mixed logical"),
        ("check", "1 < 2 < 3", "Chained comparison"),
        ("check", "1 shl 2", "Std.Bits"),
        ("check", "1 and 2", "boolean"),
        ("check", "Bits.ShiftLeft(1, true)", "integer"),
        ("run", "Bits.ShiftLeft(1, -1)", "0..63"),
        ("run", "Bits.ShiftRight(-1, 64)", "0..63"),
    ] {
        write(
            &root.join("main.fpas"),
            &format!(
                "program Main; uses Std.Bits as Bits; begin var X: integer := {expression}; end program;"
            ),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args([command, "--diagnostics", "json", "main.fpas"])
            .output()
            .expect("CLI starts");
        assert!(!output.status.success(), "{expression}: {output:?}");
        let records = stderr_records(&output);
        assert!(
            records
                .iter()
                .any(|r| r["message"].as_str().is_some_and(|s| s.contains(message))),
            "{expression}: {records:?}"
        );
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn invalid_shift_counts_report_the_numeric_domain_code_through_the_cli() {
    let root = temp_dir();
    for name in ["ShiftLeft", "ShiftRight"] {
        for count in [
            "-9223372036854775807 - 1",
            "-1",
            "64",
            "65",
            "9223372036854775807",
        ] {
            write(
                &root.join("main.fpas"),
                &format!(
                    "program Main; uses Std.Bits as Bits; begin var X: integer := Bits.{name}(0, {count}); end program;"
                ),
            );
            let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
                .current_dir(&root)
                .args(["run", "--diagnostics", "json", "main.fpas"])
                .output()
                .expect("CLI starts");
            assert_eq!(output.status.code(), Some(2), "{name}({count}): {output:?}");
            let records = stderr_records(&output);
            assert!(
                records.iter().any(|record| record["kind"] == "diagnostic"
                    && record["phase"] == "runtime"
                    && record["code"] == "F4012"
                    && record["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("0..63"))),
                "{name}({count}): {records:#?}"
            );
        }
    }
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn linked_native_runner_executes_short_circuits_and_bits() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    write(
        &root.join("main.fpas"),
        r#"program Main;
uses Std.Bits as Bits;
uses Std.Console as Console;
uses Std.Args as Args;
function Crash(): boolean;
begin panic('skipped'); return false; end function;
begin
  Console.WriteLn(false and Crash());
  Console.WriteLn(true or Crash());
  Console.WriteLn(Bits.ShiftRight(-1, 63));
  if Args.ParamCount() > 0 then Console.WriteLn(Bits.ShiftRight(0, 64)); end if;
end program;"#,
    );
    let build = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["build", "--executable", "--name", "app", "app.fpasprj"])
        .output()
        .expect("build starts");
    assert!(build.status.success(), "{build:?}");
    let executable = root.join(if cfg!(windows) { "app.exe" } else { "app" });
    let output = Command::new(&executable)
        .current_dir(&root)
        .output()
        .expect("runner starts");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "false\ntrue\n1\n");
    let invalid = Command::new(&executable)
        .current_dir(&root)
        .arg("invalid-count")
        .env("FPAS_DIAGNOSTICS", "json")
        .output()
        .expect("runner starts");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(stderr_records(&invalid).iter().any(|record| {
        record["message"]
            .as_str()
            .is_some_and(|text| text.contains("0..63"))
    }));
    fs::remove_dir_all(root).expect("remove fixtures");
}
