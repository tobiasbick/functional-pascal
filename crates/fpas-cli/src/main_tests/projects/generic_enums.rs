//! Generic enum applications and recursive payloads across fresh and reused sidecars.

use super::*;

#[test]
fn equatable_recursive_imported_enums_and_keys_survive_sidecar_reuse() {
    let cwd = create_temp_dir("equatable-recursive-enum-units");
    let project = cwd.join("enums.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/values.fpas"),
        r#"unit Data.Values;
public type Tree of T = enum Leaf(Value: T); Node(Child: option of Tree of T); Empty; end enum;
public type Bad of T = enum Full(Values: array of T); Empty; end enum;
end unit;"#,
    );
    let main = cwd.join("src/main.fpas");
    let source = r#"program T;
uses Data.Values as Values;
type Point = record X: integer; end record;
function Same<T: Equatable>(A: T; B: T): boolean; begin return A = B; end function;
function Find<K: Equatable, V>(D: dict of K to V; Key: K): option of V; begin return D.Get(Key); end function;
begin
  const A: Values.Tree of Point := Values.Tree.Node(Some(Values.Tree.Leaf(Point(X := 1))));
  const B: Values.Tree of Point := Values.Tree.Node(Some(Values.Tree.Leaf(Point(X := 2))));
  if not Same(A, A) or Same(A, B) then panic('recursive imported equality'); end if;
  var D: dict of Values.Tree of Point to string := [A: 'one', B: 'two'];
  D[A] := 'updated';
  if (Find(D, A) <> Some('updated')) or (Find(D, B) <> Some('two')) then panic('imported keys'); end if;
  const Keys := D.Keys();
  if not Same(Keys[0], A) or not Same(Keys[1], B) then panic('key order'); end if;
  const Missing: Values.Tree of Point := Values.Tree.Empty;
  if not (Find(D, Missing) is None) then panic('missing key'); end if;
end."#;
    for newline_count in 1..=2 {
        write_text(&main, &format!("{source}{}", "\n".repeat(newline_count)));
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{stderr}");
        assert!(cwd.join("src/values.fpascu").exists());
    }
    write_text(
        &main,
        r#"program T; uses Data.Values as Values;
function Same<T: Equatable>(A: T; B: T): boolean; begin return A = B; end function;
begin const X: Values.Bad of integer := Values.Bad.Empty; discard Same(X, X); end."#,
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_ne!(code, 0, "unused array payload must reject equality");
    assert!(stderr.contains("Equatable"), "{stderr}");
    assert!(!stderr.contains("FP9001"), "{stderr}");
    fs::remove_dir_all(cwd).expect("remove recursive equality fixture");
}

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
uses Data.Values as Values;
public type IntLookup = Values.Lookup of integer;
public type IntTree = Values.Tree of integer;
public type IntParcel = Values.Parcel of integer;
public type IntBox = Values.Box of integer;
public type Envelope of T = record public Value: Values.Lookup of T; end record;
public function Nested(): Values.Lookup of Values.Lookup of integer; begin return Values.Lookup.Found(Values.Lookup.Found(9)); end function;
public function Empty(): Values.Lookup of string; begin return Values.Lookup.Missing; end function;
end unit;
"#,
    );
    let main = cwd.join("src/main.fpas");
    write_text(
        &main,
        r#"
program T;
uses Data.Values as Values, Data.Facade as Facade;
var Log: string := '';
function Key(): integer; begin Log := Log + 'k'; return 7; end function;
function Text(): string; begin Log := Log + 't'; return 'seven'; end function;
begin
  const I: Facade.IntLookup := Facade.IntLookup.Found(6);
  const S: Values.Lookup of string := Values.Make('six');
  const P: Values.Pair of (integer, string) := Values.Pair.Both(Value := Text(), Key := Key());
  const TreeValue: Facade.IntTree := Facade.IntTree.Node(Right := Facade.IntTree.Empty, Left := Facade.IntTree.Leaf(3));
  const N: Values.Lookup of Values.Lookup of integer := Facade.Nested();
  const E: Facade.Envelope of integer := Facade.Envelope(Value := Values.Lookup.Found(8));
  const Missing: Values.Lookup of string := Facade.Empty();
  if Log <> 'tk' then panic('imported named order'); end if;
  if Values.First(TreeValue).Unwrap() <> 3 then panic('recursive imported enum'); end if;
  case I of when Values.Lookup.Found(const Value): if Value <> 6 then panic('integer payload'); end if; when Facade.IntLookup.Missing: panic('integer missing'); end case;
  case S of when Values.Lookup.Found(const Value): if Value <> 'six' then panic('string payload'); end if; when Values.Lookup.Missing: panic('string missing'); end case;
  case P of when Values.Pair.Both(Value := const V, Key := const K): if (K <> 7) or (V <> 'seven') then panic('named mapping'); end if; when Values.Pair.Empty: panic('pair missing'); end case;
  if N is Values.Lookup.Found(Value := Values.Lookup.Found(Value := 9)) then null; else panic('nested imported pattern'); end if;
  if E.Value is Values.Lookup.Found(8) then null; else panic('record enum payload'); end if;
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
uses Data.Facade as Facade;
begin
  const P: Facade.IntParcel := Facade.IntParcel.Packed(Facade.IntBox(Value := 7));
  case P of
    when Facade.IntParcel.Packed(const BoxValue):
    const N: integer := BoxValue.Value;
    if N <> 7 then panic('transitive record argument'); end if;
  when Facade.IntParcel.Empty: panic('transitive parcel variant'); end case;
end.
"#,
    );
    let (code, _, stderr) = support::run_cli_args_and_capture_output(
        &["run".into(), project.to_string_lossy().into_owned()],
        &cwd,
    );
    assert_eq!(code, 0, "{stderr}");
    for (statement, message) in [
        (
            "const X: Values.Lookup of string := Values.Make(1);",
            "type mismatch",
        ),
        ("discard Values.Lookup.Missing;", "Cannot infer"),
        (
            "const X: Values.Pair of (integer, string) := Values.Pair.Both(Key := 1, Wrong := 'one');",
            "Wrong",
        ),
        (
            "const X: Values.Lookup of integer := Values.Lookup.Found(1); case X of when Values.Lookup.Found(const N): begin end; end case;",
            "Missing",
        ),
    ] {
        write_text(
            &main,
            &format!("program T; uses Data.Values as Values; begin {statement} end."),
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
