//! Persisted subrange bounds and conversion diagnostics at the real CLI boundary.

use super::*;

#[test]
fn imported_subranges_survive_reuse_and_rebuild_when_bounds_change() {
    let root = temp_dir();
    write(
        &root.join("main.fpas"),
        "program T; uses Domain as D; type Same = 0..100;
        begin if not (D.ReadValue() in D.Percent) then panic('imported membership'); end if;
        const P := D.Percent(D.ReadValue()); const Copy: Same := P;
        if Copy <> 50 then panic('imported range'); end if; end.",
    );
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    let unit = root.join("Domain.fpas");
    write(
        &unit,
        "unit Domain; public const Limit: integer := 100; public type Percent = 0..Limit;
        public function ReadValue(): integer; begin return 50; end function; end unit;",
    );
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args(["run", "--diagnostics", "json", "app.fpasprj"])
            .output()
            .expect("CLI starts");
        assert!(output.status.success(), "{output:?}");
        assert!(root.join("Domain.fpascu").exists());
    }
    write(
        &unit,
        "unit Domain; public const Limit: integer := 40; public type Percent = 0..Limit;
        public function ReadValue(): integer; begin return 50; end function; end unit;",
    );
    // Keep the consumer's identity compatible so the dynamic conversion owns the failure.
    write(
        &root.join("main.fpas"),
        "program T; uses Domain as D;
        begin if D.ReadValue() in D.Percent then panic('stale membership bounds'); end if;
        const P := D.Percent(D.ReadValue()); discard P; end.",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["run", "--diagnostics", "json", "app.fpasprj"])
        .output()
        .expect("CLI starts");
    assert_eq!(output.status.code(), Some(2));
    let records = stderr_records(&output);
    assert!(
        records.iter().any(|record| record["message"]
            .as_str()
            .is_some_and(|message| message.contains("value 50") && message.contains("0..40"))),
        "{records:?}"
    );
    fs::remove_dir_all(root).expect("remove fixtures");
}

#[test]
fn static_subrange_violation_has_a_located_compile_time_diagnostic() {
    let root = temp_dir();
    write(
        &root.join("main.fpas"),
        "program T; type Percent = 0..100; begin const P := Percent(101); end.",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["check", "--diagnostics", "json", "main.fpas"])
        .output()
        .expect("CLI starts");
    let records = stderr_records(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        records.iter().any(|record| record["code"] == "FP3006"
            && !record["location"].is_null()
            && record["message"]
                .as_str()
                .is_some_and(|message| message.contains("101") && message.contains("0..100"))),
        "{records:?}"
    );
    fs::remove_dir_all(root).expect("remove fixtures");
}
