/// Tests for functions returning `Option` — `docs/pascal/07-error-handling.md`.
use super::compile_and_run;

// ── FindIndex pattern from the docs ─────────────────────────────────────

#[test]
fn find_index_returns_some() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
begin
  case FindIndex([10, 20, 30], 20) of
    Some(I): Std.Console.WriteLn('Found at ' + Std.Conv.IntToStr(I));
    None:    Std.Console.WriteLn('Not found')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Found at 1"]);
}

#[test]
fn find_index_returns_none() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
begin
  case FindIndex([10, 20, 30], 99) of
    Some(I): Std.Console.WriteLn('Found at ' + Std.Conv.IntToStr(I));
    None:    Std.Console.WriteLn('Not found')
  end
end.",
    );
    assert_eq!(out.lines, vec!["Not found"]);
}

#[test]
fn find_index_first_element() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
begin
  case FindIndex([10, 20, 30], 10) of
    Some(I): Std.Console.WriteLn(I);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["0"]);
}

#[test]
fn find_index_last_element() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
begin
  case FindIndex([10, 20, 30], 30) of
    Some(I): Std.Console.WriteLn(I);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["2"]);
}

#[test]
fn find_index_empty_array() {
    let out = compile_and_run(
        "program T;
function FindIndex(Items: array of integer; Target: integer): Option of integer;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] = Target then
      return Some(I);
  return None
end;
begin
  case FindIndex([], 5) of
    Some(I): Std.Console.WriteLn(I);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}

// ── Option of string ────────────────────────────────────────────────────

#[test]
fn option_of_string_some() {
    let out = compile_and_run(
        "program T;
function FirstNonEmpty(Items: array of string): Option of string;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] <> '' then
      return Some(Items[I]);
  return None
end;
begin
  case FirstNonEmpty(['', 'hello', 'world']) of
    Some(S): Std.Console.WriteLn(S);
    None:    Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["hello"]);
}

#[test]
fn option_of_string_none() {
    let out = compile_and_run(
        "program T;
function FirstNonEmpty(Items: array of string): Option of string;
begin
  for I: integer := 0 to Std.Array.Length(Items) - 1 do
    if Items[I] <> '' then
      return Some(Items[I]);
  return None
end;
begin
  case FirstNonEmpty(['', '', '']) of
    Some(S): Std.Console.WriteLn(S);
    None:    Std.Console.WriteLn('all empty')
  end
end.",
    );
    assert_eq!(out.lines, vec!["all empty"]);
}

// ── Option of boolean ───────────────────────────────────────────────────

#[test]
fn option_of_boolean() {
    let out = compile_and_run(
        "program T;
var O: Option of boolean := Some(true);
begin
  case O of
    Some(B): if B then Std.Console.WriteLn('yes') else Std.Console.WriteLn('no');
    None:    Std.Console.WriteLn('absent')
  end
end.",
    );
    assert_eq!(out.lines, vec!["yes"]);
}
