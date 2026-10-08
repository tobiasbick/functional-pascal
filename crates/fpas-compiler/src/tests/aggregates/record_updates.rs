use super::super::assert_succeeds;

#[test]
fn record_update_types_empty_array_override_from_the_field() {
    assert_succeeds(
        r#"
program EmptyArrayUpdate;

type
  Bag = record
    Items: array of integer;
    Names: array of string;
  end record;
begin
  const Full: Bag := Bag( Items := [1, 2, 3], Names := ['a'] );
  const Emptied: Bag := Full with Items := []; end with;
  if Emptied.Items.Length() <> 0 then panic('items not emptied'); end if;
  if Emptied.Names.Length() <> 1 then panic('names changed'); end if;
  if Full.Items.Length() <> 3 then panic('base mutated'); end if;
  const Refilled: Bag := Emptied with Items := [4]; end with;
  if Refilled.Items[0] <> 4 then panic('refill'); end if;
end.
"#,
    );
}

#[test]
fn record_update_types_context_dependent_overrides_from_the_field() {
    assert_succeeds(
        r#"
program ContextUpdate;

type
  Point = record
    X: integer;
    Y: integer;
  end record;
  type Holder = record
    Tags: dict of string to integer;
    Origin: Point;
    Label: option of string;
    Grid: array of array of integer;
    Scores: dict of string to array of integer;
  end record;
begin
  const Full: Holder := Holder(
    Tags := ['a': 1],
    Origin := Point( X := 1, Y := 2 ),
    Label := Some('named'),
    Grid := [[1], [2, 3]],
    Scores := ['a': [1]]
  );
  const Reset: Holder := Full with
    Tags := [:];
    Origin := Point( X := 7, Y := 8 );
    Label := None;
    Grid := [[]];
    Scores := ['b': []];
  end with;
  if Reset.Tags.Length() <> 0 then panic('empty dictionary'); end if;
  if Reset.Origin.X + Reset.Origin.Y <> 15 then panic('record literal'); end if;
  if not Reset.Label.IsNone() then panic('none'); end if;
  if Reset.Grid[0].Length() <> 0 then panic('nested empty array'); end if;
  if Reset.Scores['b'].Length() <> 0 then panic('empty dictionary value'); end if;
end.
"#,
    );
}

#[test]
fn record_update_propagates_context_through_option_and_result_payloads() {
    assert_succeeds(
        r#"
program NestedUpdate;

type Holder = record
  Values: option of array of integer;
  Lookup: result of dict of string to integer, string;
end record;
begin
  const Original: Holder := Holder(
    Values := Some([1]),
    Lookup := Ok(['a': 1])
  );
  const Updated: Holder := Original with
    Values := Some([]);
    Lookup := Ok([:]);
  end with;
  if Updated.Values.Unwrap().Length() <> 0 then
    panic('option payload'); end if;
  if Updated.Lookup.Unwrap().Length() <> 0 then
    panic('result payload'); end if;
  if Original.Values.Unwrap().Length() <> 1 then
    panic('base mutated'); end if;
end.
"#,
    );
}
