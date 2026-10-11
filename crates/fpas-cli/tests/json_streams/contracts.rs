//! Callee contracts survive independent compilation, interface reuse, and native bundles.
use super::*;

#[test]
fn imported_contracts_survive_reuse_indirect_calls_and_clause_changes() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"contracts\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    let unit = root.join("Api.fpas");
    write(
        &unit,
        "unit Api; public function Guard(X: integer): Result of (integer, string);
        requires X > 0; begin return Ok(X); end function;
        public type Box = record public Value: integer;
            public procedure Guard(Self: Box); requires Self.Value > 0; begin null; end procedure;
        end record; end unit;",
    );
    for call in [
        "discard A.Guard(0)",
        "const F: function(X: integer): Result of (integer, string) := A.Guard; discard F(0)",
        "const B := A.Box(Value := 0); const F: procedure() := B.Guard; F()",
    ] {
        write(
            &root.join("main.fpas"),
            &format!("program T; uses Api as A; begin {call}; end."),
        );
        for _ in 0..2 {
            let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
                .current_dir(&root)
                .args(["run", "--diagnostics", "json", "app.fpasprj"])
                .output()
                .expect("CLI starts");
            assert_eq!(output.status.code(), Some(2), "{output:?}");
            let records = stderr_records(&output);
            assert!(
                records.iter().any(|record| record["code"] == "FP5010"
                    && record["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("Api.")
                            && message.contains("requires")
                            && message.contains(": 0"))
                    && !record["location"].is_null()),
                "{records:?}"
            );
            assert!(root.join("Api.fpascu").exists());
        }
    }
    write(
        &root.join("main.fpas"),
        "program T; uses Api as A; begin discard A.Guard(1); end.",
    );
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args(["run", "--diagnostics", "json", "app.fpasprj"])
            .output()
            .expect("CLI starts")
    };
    assert!(run().status.success());
    write(
        &unit,
        "unit Api; public function Guard(X: integer): Result of (integer, string);
        requires X > 1; begin return Ok(X); end function; end unit;",
    );
    let output = run();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr_records(&output)
            .iter()
            .any(|record| record["message"]
                .as_str()
                .is_some_and(|message| message.contains("requires X > 1;"))),
        "{output:?}"
    );
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn native_bundle_checks_before_body_output_and_argument_expressions_run_once() {
    for (guard, call, stdout, failed) in [
        (
            "procedure Guard(var X: integer; Input: integer); requires X > 0; begin X := 1; WriteLn('body'); end procedure;",
            "Guard(var X, Read());",
            "argument\n",
            "requires X > 0;",
        ),
        (
            "function Returned(): integer; begin WriteLn('return'); return 0; end function;
          function Guard(var X: integer; Input: integer): integer; ensures (Value) Value > 0;
          begin X := 2; WriteLn('body'); return Returned(); end function;",
            "const Value: integer := Guard(var X, Read());",
            "argument\nbody\nreturn\n",
            "ensures (Value) Value > 0;",
        ),
    ] {
        let root = temp_dir();
        write(
            &root.join("main.fpas"),
            &format!(
                "program T; uses Std.Console;
        function Read(): integer; begin WriteLn('argument'); return 0; end function;
        {guard} begin var X := 0; {call} end."
            ),
        );
        write(
            &root.join("app.fpasprj"),
            "[project]\nname = \"contracts\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
        );
        let target = root.join(if cfg!(windows) {
            "contract.exe"
        } else {
            "contract"
        });
        let build = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args(["build", "--executable", "--name", "contract", "app.fpasprj"])
            .output()
            .expect("build starts");
        assert!(build.status.success(), "{build:?}");
        let output = Command::new(&target)
            .current_dir(&root)
            .output()
            .expect("native program starts");
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(failed),
            "{output:?}"
        );
        fs::remove_dir_all(root).expect("remove fixtures");
    }
}

#[test]
fn imported_postconditions_check_try_returns_and_rebuild_changed_clauses() {
    let root = temp_dir();
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"contracts\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    let unit = root.join("Api.fpas");
    let source = "unit Api; public function Guard(Input: Result of (integer, string)): Result of (integer, string);
        ensures (Value) case Value of when Ok(const N): N > 0; when Error(_): false; end case;
        begin const N: integer := try Input; return Ok(N); end function; end unit;";
    write(&unit, source);
    write(&root.join("main.fpas"), "program T; uses Api as A;
        begin const F: function(Input: Result of (integer, string)): Result of (integer, string) := A.Guard;
            discard F(Error('failure')); end.");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args(["run", "--diagnostics", "json", "app.fpasprj"])
            .output()
            .expect("CLI starts")
    };
    for _ in 0..2 {
        let output = run();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        let records = stderr_records(&output);
        assert!(
            records.iter().any(|record| record["code"] == "FP5010"
                && !record["location"].is_null()
                && record["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("Api.Guard")
                        && message.contains("ensures (Value)")
                        && message.contains("Return: Error(failure)"))),
            "{records:?}"
        );
    }
    assert!(root.join("Api.fpascu").exists());
    write(
        &unit,
        &source.replace("when Error(_): false", "when Error(_): true"),
    );
    assert!(run().status.success());
    fs::remove_dir_all(root).expect("remove fixtures");
}
