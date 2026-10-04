//! Structural `=` and `<>` on records and payload enums at runtime.

use super::super::assert_succeeds;

#[test]
fn collections_compare_structurally_and_dictionary_order_is_irrelevant() {
    assert_succeeds(
        "program Main;\n        function Same of (T: Equatable)(A: T; B: T): boolean;\n        begin return A = B; end function;\n        begin\n          if not Same([[1], [2]], [[1], [2]]) then panic('nested arrays'); end if;\n          if Same([1, 2], [2, 1]) then panic('array order'); end if;\n          if not Same(['a': [1], 'b': [2]], ['b': [2], 'a': [1]]) then panic('dict order'); end if;\n          if Same(['a': [1]], ['a': [2]]) then panic('dict values'); end if;\n          if Same(['a': [1]], ['b': [1]]) then panic('dict keys'); end if;\n        end program;",
    );
}

#[test]
fn equality_inference_completes_empty_constructor_and_collection_components() {
    assert_succeeds(
        "program Main;\n         function Same of (T: Equatable)(A: T; B: T): boolean;\n         begin return A = B; end function;\n         begin\n           if Same(Option.None, Option.Some(1)) or Same(Option.Some(1), Option.None) then panic('option'); end if;\n           if Same(Result.Ok(1), Result.Error('failed')) or Same(Result.Error('failed'), Result.Ok(1)) then panic('result'); end if;\n           if not Same([Option.None, Option.Some(1)], [Option.None, Option.Some(1)]) then panic('array'); end if;\n           if not Same(['a': Option.None, 'b': Option.Some(1)], ['b': Option.Some(1), 'a': Option.None]) then panic('dict'); end if;\n           if Same([], [[Option.None, Option.Some(1)]]) then panic('empty array'); end if;\n         end program;",
    );
}

#[test]
fn scalar_and_nested_real_equality_share_nan_and_signed_zero_rules() {
    assert_succeeds(
        r#"program Main;

uses Std.Math as Math;
uses Std.Arrays as Arrays;

type Data = record
  Value: real;
end record;

begin
  const Nan: real := Math.Pow(-1.0, 0.5);
  const Wrapped: Option of (real) := Option.Some(Nan);
  const RecordValue: Data := Data(Value := Nan);
  if (Nan = Nan) or (Wrapped = Wrapped) or (RecordValue = RecordValue) or ([Nan] = [Nan]) or
     (['a': Nan] = ['a': Nan]) then
    panic('NaN equality');
  end if;

  if not ([0.0] = [-0.0]) then
    panic('signed zero');
  end if;

  if not (['a': 0.0] = ['a': -0.0]) then
    panic('dict signed zero');
  end if;

  if (Nan in [Nan]) or Arrays.Contains([Nan], Nan) then
    panic('NaN membership');
  end if;

  if Arrays.IndexOf([Nan], Nan) <> -1 then
    panic('NaN index');
  end if;
end program;
"#,
    );
}

#[test]
fn dictionary_keys_use_structural_equality_for_lookup_updates_and_helpers() {
    assert_succeeds(
        r#"program Main; uses Std.Dictionaries as Dictionaries; uses Std.Options as Options;
        begin
          const Key: dict of (string, integer) := ['a': 1, 'b': 2];
          const Reordered: dict of (string, integer) := ['b': 2, 'a': 1];
           var Values: dict of (dict of (string, integer), integer) := [Key: 1];
          if Values[Reordered] <> 1 then panic('lookup'); end if;
          Values[Reordered] := 42;
          if Dictionaries.Length(Values) <> 1 then panic('update appended'); end if;
          if not (Reordered in Values) then panic('membership'); end if;
          if not Dictionaries.ContainsKey(Values, Reordered) then panic('contains'); end if;
          if Options.Unwrap(Dictionaries.Get(Values, Reordered)) <> 42 then panic('get'); end if;
          const Merged: dict of (dict of (string, integer), integer) := Dictionaries.Merge(Values, [Reordered: 7]);
          if (Dictionaries.Length(Merged) <> 1) or (Merged[Key] <> 7) then panic('merge'); end if;
          if Dictionaries.Length(Dictionaries.Remove(Values, Reordered)) <> 0 then panic('remove'); end if;
        end program;"#,
    );
}

#[test]
fn records_and_payload_enums_compare_structurally() {
    assert_succeeds(
        r#"program Eq2;

type Point = record
  X: integer;
  Y: real;
end record;

type Box = record
  Corner: Point;
  Label: string;
  Tag: Option of (Point);
end record;

type Shape = enum
  Circle(Center: Point; Radius: integer);
  Square(Side: integer);
  Dot;
end enum;

procedure Check(Name: string; Value: boolean; Expected: boolean);
begin
  if Value <> Expected then
    panic('wrong: ' + Name);
  end if;
end procedure;

begin
  const A: Point := Point(X := 1, Y := 2.0);
  const B: Point := Point(X := 1, Y := 2.0);
  const C: Point := A with Y := 2.5; end with;

  Check('same fields', A = B, true);
  Check('updated field', A = C, false);
  Check('not equal', A <> C, true);
  const Box1: Box := Box(Corner := A, Label := 'a', Tag := Option.Some(B));
  const Box2: Box := Box(Corner := B, Label := 'a', Tag := Option.Some(A));
  Check('nested', Box1 = Box2, true);
  Check('nested differs', Box1 = (Box2 with Tag := Option.None; end with), false);
  const S1: Shape := Shape.Circle(A, 3);
  Check('same variant payload', S1 = Shape.Circle(B, 3), true);
  Check('same variant other payload', S1 = Shape.Circle(A, 4), false);
  Check('other variant', S1 = Shape.Square(3), false);
  Check('unit variant', Shape.Dot = Shape.Dot, true);
  Check('unit vs payload', Shape.Dot <> S1, true);
end program;
"#,
    );
}
