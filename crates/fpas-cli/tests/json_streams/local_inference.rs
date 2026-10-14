//! Real CLI annotation diagnostics and inferred types through compiled units.

use super::*;

#[test]
fn underdetermined_local_inference_reports_a_located_annotation_hint() {
    let root = temp_dir();
    write(
        &root.join("main.fpas"),
        "program T; begin const Value := None; end.",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["check", "--diagnostics", "json", "main.fpas"])
        .output()
        .expect("CLI starts");
    fs::remove_dir_all(root).expect("remove fixtures");
    assert_eq!(output.status.code(), Some(1));
    let records = stderr_records(&output);
    assert!(
        records.iter().any(|record| record["code"] == "FP3036"
            && !record["location"].is_null()
            && record["hint"]
                .as_str()
                .is_some_and(|hint| hint.contains("const Value: Type :="))),
        "{records:?}"
    );
}

#[test]
fn imported_constructions_conversions_and_private_inference_survive_unit_reuse() {
    let root = temp_dir();
    write(
        &root.join("main.fpas"),
        "program T; uses Domain as D; begin const Id := D.UserId(42); const Raw := integer(Id); const Boxed := D.Box(Value := D.Lookup.Found(Raw)); const Missing := D.IntLookup.Missing; const Read := D.Read(); const Empty: Option of integer := D.Empty(); const Factory: function(): Option of string := D.Empty; if Raw <> 42 or Read <> 3 or not (Empty is None) or not (Factory() is None) then panic('import inference'); end if; case Boxed.Value of when D.Lookup.Found(const Value): if Value <> 42 then panic('payload'); end if; when D.Lookup.Missing: panic('missing'); end case; case Missing of when D.Lookup.Found(_): panic('found'); when D.Lookup.Missing: null; end case; end.",
    );
    write(
        &root.join("app.fpasprj"),
        "[project]\nname = \"app\"\nkind = \"program\"\nmain = \"main.fpas\"\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    let unit = root.join("Domain.fpas");
    let declarations = "unit Domain; public type UserId = distinct integer; public type Box of T = record public Value: T; end record; public type Lookup of T = enum Found(Value: T); Missing; end enum; public type IntLookup = Lookup of integer; public function Empty<T>(): Option of T; begin return None; end function; function Number(): integer; begin return 3; end function;";
    write(
        &unit,
        &format!(
            "{declarations} public function Read(): integer; begin const Private := Box(Value := Number()); return Private.Value; end function; end unit;"
        ),
    );
    for _ in 0..2 {
        let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
            .current_dir(&root)
            .args(["run", "--diagnostics", "json", "app.fpasprj"])
            .output()
            .expect("CLI starts");
        assert!(output.status.success(), "{output:?}");
        assert!(root.join("Domain.fpascu").exists(), "compiled unit sidecar");
    }
    write(
        &unit,
        &format!(
            "{declarations} public function Read(): integer; begin const Private := Lookup.Found(3); return case Private of when Lookup.Found(const Value): Value; when Lookup.Missing: 0; end case; end function; end unit;"
        ),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(&root)
        .args(["run", "--diagnostics", "json", "app.fpasprj"])
        .output()
        .expect("CLI starts");
    fs::remove_dir_all(root).expect("remove fixtures");
    assert!(output.status.success(), "{output:?}");
}
