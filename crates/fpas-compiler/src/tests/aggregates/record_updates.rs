use super::super::assert_succeeds;

#[test]
fn record_update_types_empty_array_override_from_the_field() {
    assert_succeeds(
        r#"program EmptyArrayUpdate;

uses Std.Arrays as Arrays;

type Bag = record
  Items: array of (integer);
  Names: array of (string);
end record;

begin
  const Full: Bag := Bag(Items := [1, 2, 3], Names := ['a']);
  const Emptied: Bag := Full with Items := []; end with;

  if Arrays.Length(Emptied.Items) <> 0 then
    panic('items not emptied');
  end if;

  if Arrays.Length(Emptied.Names) <> 1 then
    panic('names changed');
  end if;

  if Arrays.Length(Full.Items) <> 3 then
    panic('base mutated');
  end if;
  const Refilled: Bag := Emptied with Items := [4]; end with;

  if Refilled.Items[0] <> 4 then
    panic('refill');
  end if;
end program;
"#,
    );
}

#[test]
fn record_update_types_context_dependent_overrides_from_the_field() {
    assert_succeeds(
        r#"program ContextUpdate;

uses Std.Arrays as Arrays;
uses Std.Dictionaries as Dictionaries;
uses Std.Options as Options;

type Point = record
  X: integer;
  Y: integer;
end record;

type Holder = record
  Tags: dict of (string, integer);
  Origin: Point;
  Label: Option of (string);
  Grid: array of (array of (integer));
  Scores: dict of (string, array of (integer));
end record;

begin
  const Full: Holder := Holder(Tags := ['a': 1], Origin := Point(X := 1, Y := 2), Label := Option.Some('named'), Grid := [
                                                                                                                         [1],
                                                                                                                         [2, 3]
                                                                                                                       ], Scores := ['a': [
                                                                                                                                            1
                                                                                                                                          ]]);
  const Reset: Holder := Full with Tags := [:]; Origin := Point(X := 7, Y := 8); Label := Option.None; Grid := [
                                                                                                               []
                                                                                                             ]; Scores := ['b': []]; end with;

  if Dictionaries.Length(Reset.Tags) <> 0 then
    panic('empty dictionary');
  end if;

  if Reset.Origin.X + Reset.Origin.Y <> 15 then
    panic('record literal');
  end if;

  if not Options.IsNone(Reset.Label) then
    panic('none');
  end if;

  if Arrays.Length(Reset.Grid[0]) <> 0 then
    panic('nested empty array');
  end if;

  if Arrays.Length(Reset.Scores['b']) <> 0 then
    panic('empty dictionary value');
  end if;
end program;
"#,
    );
}

#[test]
fn record_update_propagates_context_through_option_and_result_payloads() {
    assert_succeeds(
        r#"program NestedUpdate;

uses Std.Arrays as Arrays;
uses Std.Dictionaries as Dictionaries;
uses Std.Options as Options;
uses Std.Results as Results;

type Holder = record
  Values: Option of (array of (integer));
  Lookup: Result of (dict of (string, integer), string);
end record;

begin
  const Original: Holder := Holder(Values := Option.Some([1]), Lookup := Result.Ok(['a': 1]));
  const Updated: Holder := Original with Values := Option.Some([]); Lookup := Result.Ok([:]); end with;

  if Arrays.Length(Options.Unwrap(Updated.Values)) <> 0 then
    panic('option payload');
  end if;

  if Dictionaries.Length(Results.Unwrap(Updated.Lookup)) <> 0 then
    panic('result payload');
  end if;

  if Arrays.Length(Options.Unwrap(Original.Values)) <> 1 then
    panic('base mutated');
  end if;
end program;
"#,
    );
}
