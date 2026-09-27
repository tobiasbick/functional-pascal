use super::super::assert_succeeds;

#[test]
fn record_update_types_empty_array_override_from_the_field() {
    assert_succeeds(
        r#"
program EmptyArrayUpdate;
uses Std.Arrays;
type
  Bag = record
    Items: array of integer;
    Names: array of string;
  end;
begin
  var Full: Bag := record Items := [1, 2, 3]; Names := ['a']; end;
  var Emptied: Bag := Full with Items := []; end;
  if Std.Arrays.Length(Emptied.Items) <> 0 then panic('items not emptied');
  if Std.Arrays.Length(Emptied.Names) <> 1 then panic('names changed');
  if Std.Arrays.Length(Full.Items) <> 3 then panic('base mutated');
  var Refilled: Bag := Emptied with Items := [4]; end;
  if Refilled.Items[0] <> 4 then panic('refill')
end.
"#,
    );
}

#[test]
fn record_update_types_context_dependent_overrides_from_the_field() {
    assert_succeeds(
        r#"
program ContextUpdate;
uses Std.Arrays, Std.Dictionaries, Std.Options;
type
  Point = record
    X: integer;
    Y: integer;
  end;
  Holder = record
    Tags: dict of string to integer;
    Origin: Point;
    Label: option of string;
    Grid: array of array of integer;
    Scores: dict of string to array of integer;
  end;
begin
  var Full: Holder := record
    Tags := ['a': 1];
    Origin := record X := 1; Y := 2; end;
    Label := Some('named');
    Grid := [[1], [2, 3]];
    Scores := ['a': [1]];
  end;
  var Reset: Holder := Full with
    Tags := [:];
    Origin := record X := 7; Y := 8; end;
    Label := None;
    Grid := [[]];
    Scores := ['b': []];
  end;
  if Std.Dictionaries.Length(Reset.Tags) <> 0 then panic('empty dictionary');
  if Reset.Origin.X + Reset.Origin.Y <> 15 then panic('record literal');
  if not Std.Options.IsNone(Reset.Label) then panic('none');
  if Std.Arrays.Length(Reset.Grid[0]) <> 0 then panic('nested empty array');
  if Std.Arrays.Length(Reset.Scores['b']) <> 0 then panic('empty dictionary value')
end.
"#,
    );
}

#[test]
fn record_update_propagates_context_through_option_and_result_payloads() {
    assert_succeeds(
        r#"
program NestedUpdate;
uses Std.Arrays, Std.Dictionaries, Std.Options, Std.Results;
type Holder = record
  Values: option of array of integer;
  Lookup: result of dict of string to integer, string;
end;
begin
  var Original: Holder := record
    Values := Some([1]);
    Lookup := Ok(['a': 1]);
  end;
  var Updated: Holder := Original with
    Values := Some([]);
    Lookup := Ok([:]);
  end;
  if Std.Arrays.Length(Std.Options.Unwrap(Updated.Values)) <> 0 then
    panic('option payload');
  if Std.Dictionaries.Length(Std.Results.Unwrap(Updated.Lookup)) <> 0 then
    panic('result payload');
  if Std.Arrays.Length(Std.Options.Unwrap(Original.Values)) <> 1 then
    panic('base mutated')
end.
"#,
    );
}
