//! Pattern extraction retains callable capabilities without changing evaluation.
//!
//! Documentation: `docs/pascal/language/functions/discard.md`

use super::assert_succeeds;

#[test]
fn task_free_pattern_callables_can_be_spawned_and_discarded() {
    assert_succeeds(
        r#"
program PatternTasks;
uses Std.Tasks;
function Work(): integer;
begin
  return 42;
end function;
procedure Main();
begin
  const Wrapped: Option of function(): integer := Some(Work);
  var Sum: integer := 0;
  if Wrapped is Some(const F) then
    discard F;
    const Alias: function(): integer := function(): integer begin
      return F();
    end function;
    discard Alias;
    const Job: task := go Alias();
    Sum := Sum + Wait(Job);
  end if;
  while Wrapped is Some(const F) do
    discard F;
    const Job: task := go F();
    Sum := Sum + Wait(Job);
    break;
  end while;
  case Some(Ok(Wrapped)) of
    when Some(Ok(Some(const F))):
      discard F;
      const Job: task := go F();
      Sum := Sum + Wait(Job);
    else
      panic('missing payload');
  end case;
  if Sum <> 126 then panic('pattern task result'); end if;
end procedure;
begin
  Main();
end.
"#,
    );
}

#[test]
fn discarding_pattern_callables_never_invokes_them() {
    assert_succeeds(
        r#"
program PatternDiscard;
procedure Main();
begin
  var Calls: integer := 0;
  const Change: procedure() := procedure() begin
    Calls := Calls + 1;
  end procedure;
  const Wrapped: Option of procedure() := Some(Change);
  if Wrapped is Some(const F) then discard F; end if;
  while Wrapped is Some(const F) do
    discard F;
    break;
  end while;
  case Wrapped of
    when Some(const F): discard F;
    when None: panic('missing payload');
  end case;
  if Calls <> 0 then panic('discard invoked callable'); end if;
  Change();
  if Calls <> 1 then panic('same-task callable invocation'); end if;
end procedure;
begin
  Main();
end.
"#,
    );
}

#[test]
fn scalar_fields_of_task_bound_pattern_values_can_be_captured_and_spawned() {
    assert_succeeds(
        r#"
program PatternScalar;
uses Std.Tasks;
type WorkBox = record
  Change: procedure();
  Number: integer;
end record;
procedure Main();
begin
  var Calls: integer := 0;
  const Change: procedure() := procedure() begin Calls := Calls + 1; end procedure;
  const Wrapped: Option of WorkBox := Some(WorkBox(Change := Change, Number := 42));
  if Wrapped is Some(const Box) then
    discard Box.Change;
    const N: integer := Box.Number;
    const GetNumber: function(): integer := function(): integer begin
      return N;
    end function;
    const Job: task := go GetNumber();
    if Wait(Job) <> 42 then panic('scalar capture'); end if;
    Box.Change();
  end if;
  if Calls <> 1 then panic('record callable'); end if;
end procedure;
begin
  Main();
end.
"#,
    );
}

#[test]
fn closures_capture_distinct_pattern_bindings_from_the_same_label() {
    assert_succeeds(
        r#"
program MultiplePatternCaptures;
uses Std.Tasks;
type Pair = enum
  Values(First: integer; Second: integer);
end enum;
procedure Main();
begin
  const Wrapped: Pair := Pair.Values(20, 22);
  case Wrapped of
    when Pair.Values(const A, const B):
      const Add: function(): integer := function(): integer begin
        return A + B;
      end function;
      discard Add;
      const Job: task := go Add();
      if Wait(Job) <> 42 then panic('distinct captures'); end if;
  end case;
end procedure;
begin
  Main();
end.
"#,
    );
}
