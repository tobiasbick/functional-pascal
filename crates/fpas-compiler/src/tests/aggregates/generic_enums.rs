//! Generic enum erasure, concrete pattern payloads, recursion, and written order.

use super::*;

#[test]
fn generic_payloads_nested_patterns_and_recursive_routines_execute() {
    assert_succeeds(
        r#"
program T;
type Lookup of T = enum Found(Value: T); Missing; end enum;
type Tree of T = enum Leaf(Value: T); Node(Left: Tree of T; Right: Tree of T); Empty; end enum;
type Box of T = record Value: T; end record;
type IntLookup = Lookup of integer;
type Choice of (L, R) = enum Left(Value: L); Right(Value: R); Neither; end enum;
type Mapper of (T, R) = enum Build(Input: T; Transform: function(Value: T): R); Empty; end enum;
function GenericText<T>(Value: T): string; begin return 'generic'; end function;
function Apply(Value: Mapper of (integer, string)): string;
begin return case Value of when Mapper.Build(const Input, const Transform): Transform(Input); when Mapper.Empty: ''; end case; end function;
function Read(Value: Lookup of integer): integer;
begin return case Value of when Lookup.Found(const N): N + 1; when Lookup.Missing: 0; end case; end function;
function ReadText(Value: Lookup of string): string;
begin return case Value of when Lookup.Found(const S): S + '!'; when Lookup.Missing: ''; end case; end function;
function Sum(Value: Tree of integer): integer;
begin return case Value of when Tree.Leaf(const N): N; when Tree.Node(const L, const R): Sum(L) + Sum(R); when Tree.Empty: 0; end case; end function;
function First<T>(Value: Tree of T): Option of T;
begin return case Value of when Tree.Leaf(const Item): Some(Item); when Tree.Node(const L, _): First(L); when Tree.Empty: None; end case; end function;
function Make<T>(Value: T): Lookup of T; begin return Lookup.Found(Value); end function;
procedure Accept<T>(Value: Lookup of T; Evidence: T); begin if Value is Lookup.Found(const Item) then panic('expected missing'); end if; end procedure;
begin
  const I: IntLookup := IntLookup.Found(6);
  const S: Lookup of string := Make('six');
  const Root: Tree of integer := Tree.Node(Right := Tree.Empty, Left := Tree.Node(Tree.Leaf(3), Tree.Leaf(4)));
  if (Read(I) <> 7) or (ReadText(S) <> 'six!') or (Sum(Root) <> 7) or (First(Root).Unwrap() <> 3) then panic('generic enum routines'); end if;
  if Lookup.Found(6) <> I then panic('generic equality'); end if;
  Accept(Lookup.Missing, 1);
  Accept(Evidence := 'x', Value := Lookup.Missing);
  const Nested: Lookup of Result of (Option of Box of integer, string) := Lookup.Found(Ok(Some(Box(Value := 9))));
  case Nested of
    when Lookup.Found(Ok(Some(const B))): if B.Value <> 9 then panic('concrete nested payload'); end if;
    when Lookup.Found(Ok(None)): panic('unexpected None');
    when Lookup.Found(Error(_)): panic('unexpected Error');
    when Lookup.Missing: panic('unexpected Missing');
  end case;
  const NestedEnum: Lookup of Lookup of integer := Lookup.Found(Lookup.Found(2));
  case NestedEnum of
    when Lookup.Found(Lookup.Found(2)): begin end;
    when Lookup.Found(Lookup.Found(const N)): panic('literal mismatch');
    when Lookup.Found(Lookup.Missing): panic('nested missing');
    when Lookup.Missing: panic('outer missing');
  end case;
  const Partial: Choice of (integer, string) := Choice.Left(5);
  case Partial of when Choice.Left(const L): if L <> 5 then panic('partial inference'); end if; when Choice.Right(_): panic('right'); when Choice.Neither: panic('neither'); end case;
  const Numbers: Lookup of array of integer := Lookup.Found([1, 2]);
  if Numbers is Lookup.Found(const Values) and Values[1] <> 2 then panic('array payload'); end if;
  if Apply(Mapper.Build(Input := 1, Transform := GenericText)) <> 'generic' then panic('generic callable payload'); end if;
  if Apply(Mapper.Build(Transform := GenericText, Input := 1)) <> 'generic' then panic('reordered generic callable payload'); end if;
end.
"#,
    );
}

#[test]
fn named_generic_variants_evaluate_payloads_once_in_written_order() {
    assert_succeeds(
        r#"
program T;
type Pair of (K, V) = enum Both(Key: K; Value: V); Empty; end enum;
var Log: string := '';
function Key(): integer; begin Log := Log + 'k'; return 7; end function;
function Value(): string; begin Log := Log + 'v'; return 'seven'; end function;
function KeyOf<K, V>(Value: Pair of (K, V)): K;
begin case Value of when Pair.Both(const Key, _): return Key; when Pair.Empty: panic('empty key'); end case; end function;
begin
  const P: Pair of (integer, string) := Pair.Both(Value := Value(), Key := Key());
  if Log <> 'vk' then panic('written order'); end if;
  case P of when Pair.Both(const K, const V): if (K <> 7) or (V <> 'seven') then panic('field mapping'); end if; when Pair.Empty: panic('empty'); end case;
  Log := '';
  const Inferred: integer := KeyOf(Pair.Both(Value := Value(), Key := Key()));
  if (Inferred <> 7) or (Log <> 'vk') then panic('named inference without an annotation'); end if;
end.
"#,
    );
}

#[test]
fn mutually_recursive_enums_and_records_execute() {
    assert_succeeds(
        r#"
program T;
type A of T = enum Stop; Next(Value: B of T); end enum;
type B of U = enum Stop; Next(Value: A of U); end enum;
type Entry of T = record Value: T; Next: Link of T; end record;
type Link of U = enum Stop; More(Value: Entry of U); end enum;
begin
  const Root: A of integer := A.Next(B.Next(A.Stop));
  if Root is A.Next(B.Next(A.Stop)) then begin end; else panic('mutual enum pattern'); end if;
  const E: Entry of integer := Entry(Value := 2, Next := Link.More(Entry(Value := 1, Next := Link.Stop)));
  if E.Next is Link.More(const Tail) and Tail.Value = 1 then begin end; else panic('record enum recursion'); end if;
end.
"#,
    );
}
