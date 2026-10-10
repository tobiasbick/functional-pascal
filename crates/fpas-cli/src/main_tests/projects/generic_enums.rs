//! Generic enum applications and recursive payloads across fresh and reused sidecars.

use super::*;

#[test]
fn imported_generic_enums_preserve_payloads_aliases_and_named_order() {
    let cwd = create_temp_dir("generic-enum-units");
    let project = cwd.join("enums.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/values.fpas"),
        r#"
unit Data.Values;
type T = string;
public type Lookup of T = enum Found(Value: T); Missing; end enum;
public type Pair of (K, V) = enum Both(Key: K; Value: V); Empty; end enum;
public type Tree of T = enum Leaf(Value: T); Node(Left: Tree of T; Right: Tree of T); Empty; end enum;
public type Box of T = record public Value: T; end record;
public type Parcel of T = enum Packed(Item: Box of T); Empty; end enum;
public function Make<T>(Value: T): Lookup of T; begin return Lookup.Found(Value); end function;
public function First<T>(Value: Tree of T): Option of T;
begin return case Value of when Tree.Leaf(const Item): Some(Item); when Tree.Node(const Left, _): First(Left); when Tree.Empty: None; end case; end function;
end unit;
"#,
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        r#"
unit Data.Facade;
uses Data.Values;
public type IntLookup = Lookup of integer;
public type IntTree = Tree of integer;
public type IntParcel = Parcel of integer;
public type IntBox = Box of integer;
public type Envelope of T = record public Value: Lookup of T; end record;
public function Nested(): Lookup of Lookup of integer; begin return Lookup.Found(Lookup.Found(9)); end function;
public function Empty(): Lookup of string; begin return Lookup.Missing; end function;
end unit;
"#,
    );
    let main = cwd.join("src/main.fpas");
    write_text(
        &main,
        r#"
program T;
uses Data.Values as Values, Data.Facade;
var Log: string := '';
function Key(): integer; begin Log := Log + 'k'; return 7; end function;
function Text(): string; begin Log := Log + 't'; return 'seven'; end function;
begin
  const I: IntLookup := IntLookup.Found(6);
  const S: Values.Lookup of string := Values.Make('six');
  const P: Values.Pair of (integer, string) := Values.Pair.Both(Value := Text(), Key := Key());
  const TreeValue: IntTree := IntTree.Node(Right := IntTree.Empty, Left := IntTree.Leaf(3));
  const N: Values.Lookup of Values.Lookup of integer := Nested();
  const E: Envelope of integer := Envelope(Value := Values.Lookup.Found(8));
  const Missing: Values.Lookup of string := Data.Facade.Empty();
  if Log <> 'tk' then panic('imported named order'); end if;
  if Values.First(TreeValue).Unwrap() <> 3 then panic('recursive imported enum'); end if;
  case I of when Values.Lookup.Found(const Value): if Value <> 6 then panic('integer payload'); end if; when IntLookup.Missing: panic('integer missing'); end case;
  case S of when Values.Lookup.Found(const Value): if Value <> 'six' then panic('string payload'); end if; when Values.Lookup.Missing: panic('string missing'); end case;
  case P of when Values.Pair.Both(const K, const V): if (K <> 7) or (V <> 'seven') then panic('named mapping'); end if; when Values.Pair.Empty: panic('pair missing'); end case;
  if N is Values.Lookup.Found(Values.Lookup.Found(const Value)) and Value = 9 then begin end; else panic('nested imported pattern'); end if;
  if E.Value is Values.Lookup.Found(const Value) and Value = 8 then begin end; else panic('record enum payload'); end if;
  if Missing is Values.Lookup.Missing then begin end; else panic('fieldless imported enum'); end if;
end.
"#,
    );
    for _ in 0..2 {
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{stderr}");
        assert!(cwd.join("src/values.fpascu").exists(), "unit sidecar");
    }
    write_text(
        &main,
        r#"
program T;
uses Data.Facade;
begin
  const P: IntParcel := IntParcel.Packed(IntBox(Value := 7));
  if P is IntParcel.Packed(const BoxValue) then
    const N: integer := BoxValue.Value;
    if N <> 7 then panic('transitive record argument'); end if;
  else panic('transitive parcel variant'); end if;
end.
"#,
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["run".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(code, 0, "{stderr}");
    for (statement, message) in [
        ("const X: Lookup of string := Make(1);", "type mismatch"),
        ("discard Lookup.Missing;", "Cannot infer"),
        (
            "const X: Pair of (integer, string) := Pair.Both(Key := 1, Wrong := 'one');",
            "Wrong",
        ),
        (
            "const X: Lookup of integer := Lookup.Found(1); case X of when Lookup.Found(const N): begin end; end case;",
            "Missing",
        ),
    ] {
        write_text(
            &main,
            &format!("program T; uses Data.Values; begin {statement} end."),
        );
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["check".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 1, "{stderr}");
        assert!(
            stderr
                .to_ascii_lowercase()
                .contains(&message.to_ascii_lowercase()),
            "{stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("remove project fixture");
}
