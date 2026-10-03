use super::super::assert_succeeds;

#[test]
fn copied_stateful_closures_share_identity_and_named_captures_match() {
    assert_succeeds(
        r#"
program Copies;
type Counter = function(): integer;
function Anonymous(): Counter;
begin
  mutable var Count: integer := 0;
  return function(): integer begin Count := Count + 1; return Count; end function;
end function;
function Named(mutable Count: integer): Counter;
  function Next(): integer; begin Count := Count + 1; return Count; end function;
begin return Next; end function;
begin
  var First: Counter := Anonymous();
  var Copy: Counter := First;
  if First() <> 1 then panic('first'); end if;
  if (Copy)() <> 2 then panic('copy identity'); end if;
  var Other: Counter := Named(0);
  var Copies: array of Counter := [Other, Other];
  if Copies[0]() <> 1 then panic('named first'); end if;
  if Copies[1]() <> 2 then panic('named identity'); end if;
  if Named(0)() <> 1 then panic('separate activation'); end if;
end program;
"#,
    );
}

#[test]
fn mutable_parameter_cells_are_local_and_shared_with_anonymous_captures() {
    assert_succeeds(
        r#"
program ParameterCells;
function Make(mutable Count: integer): function(): integer;
begin
  var Next: function(): integer := function(): integer
  begin Count := Count + 1; return Count; end function;
  Count := Count + 1;
  return Next;
end function;
begin
  var Original: integer := 40;
  var Next: function(): integer := Make(Original);
  if Next() <> 42 then panic('shared parameter cell'); end if;
  if Original <> 40 then panic('caller parameter snapshot'); end if;
end program;
"#,
    );
}

#[test]
fn captures_in_targets_indices_and_arguments_respect_shadowing() {
    assert_succeeds(
        r#"
program CaptureTraversal;
type Handler = function(X: integer): integer;
function Make(): function(): integer;
begin
  var Functions: array of Handler := [function(X: integer): integer begin return X + 1; end function];
  var Index: integer := 0;
  var Value: integer := 41;
  return function(): integer
  begin
    begin var Value: integer := 9; discard Functions[Index](Value); end;
    return (Functions[Index])(Value);
  end function;
end function;
begin if Make()() <> 42 then panic('capture traversal'); end if; end program;
"#,
    );
}

#[test]
fn loop_captures_keep_each_iteration_value() {
    assert_succeeds(
        r#"
program LoopCaptures;
type Getter = function(): integer;
function Zero(): integer; begin return 0; end function;
procedure Check();
begin
  mutable var Functions: array of Getter := [Zero, Zero, Zero];
  for I: integer := 0 to 2 do
    Functions[I] := function(): integer begin return I; end function;
  end for;
  if Functions[0]() <> 0 then panic('iteration zero'); end if;
  if Functions[1]() <> 1 then panic('iteration one'); end if;
  if Functions[2]() <> 2 then panic('iteration two'); end if;
end procedure;
begin Check(); end program;
"#,
    );
}

#[test]
fn immutable_nested_value_capture_is_a_snapshot() {
    assert_succeeds(
        r#"
program CaptureSnapshot;
procedure Check();
begin
  mutable var Original: array of array of integer := [[42]];
  var Snapshot: array of array of integer := Original;
  var GetSnapshot: function(): integer := function(): integer begin return Snapshot[0][0]; end function;
  Original[0][0] := 99;
  if GetSnapshot() <> 42 then panic('immutable nested capture snapshot'); end if;
end procedure;
begin Check(); end program;
"#,
    );
}
