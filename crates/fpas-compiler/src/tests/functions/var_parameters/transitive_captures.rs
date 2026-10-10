//! Named sibling calls preserve the storage of their transitive captures.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`

use super::assert_succeeds;

#[test]
fn recursive_anonymous_wrappers_keep_value_captures_after_the_parent_returns() {
    assert_succeeds(
        r#"
program RecursiveWrappers;
function Make(Base: integer): function(): integer;
  function First(Depth: integer): integer;
  begin
    const Next: function(): integer := function(): integer begin
      return First(Depth - 1);
    end function;
    if Depth > 0 then return Next() + 1; end if;
    return Base;
  end function;
begin
  return function(): integer begin return First(2); end function;
end function;
begin
  const Invoke: function(): integer := Make(40);
  if Invoke() <> 42 then panic('recursive wrapper value capture'); end if;
end.
"#,
    );
}

#[test]
fn recursive_wrappers_preserve_capture_shadowing_and_shared_mutable_cells() {
    assert_succeeds(
        r#"
program RecursiveCaptureIdentity;
function Make(Value: string): function(): integer;
  function First(Depth: integer): integer;
    function Bridge(Value: integer): integer;
    begin
      const Next: function(): integer := function(): integer begin
        if Value <> 7 then panic('nearest capture'); end if;
        return First(Depth - 1);
      end function;
      return Next();
    end function;
  begin
    if Depth > 0 then return Bridge(7); end if;
    if Value <> 'outer' then panic('outer capture'); end if;
    Count := Count + 1;
    return Count;
  end function;
begin
  var Count: integer := 0;
  return function(): integer begin return First(2); end function;
end function;
begin
  const Invoke: function(): integer := Make('outer');
  const Copy: function(): integer := Invoke;
  if Invoke() <> 1 then panic('first shared write'); end if;
  if Copy() <> 2 then panic('second shared write'); end if;
end.
"#,
    );
}

#[test]
fn sibling_chains_forward_reference_captures_and_allow_recursion() {
    assert_succeeds(
        r#"
program SiblingReferences;
procedure Outer(var Value: integer);
  procedure First();
  begin
    Value := Value + 1;
  end procedure;
  procedure Second();
  begin
    First();
  end procedure;
  procedure Third(Count: integer);
  begin
    if Count > 0 then
      Second();
      Third(Count - 1);
    end if;
  end procedure;
begin
  Third(3);
end procedure;
begin
  var Total: integer := 0;
  Outer(var Total);
  if Total <> 3 then panic('transitive reference'); end if;
end.
"#,
    );
}

#[test]
fn sibling_captures_keep_their_identity_under_parameter_and_local_shadowing() {
    assert_succeeds(
        r#"
program ShadowedSiblingReferences;
procedure Outer(var Value: integer);
  procedure First();
  begin
    Value := Value + 1;
  end procedure;
  procedure Second(Value: integer);
  begin
    First();
    if Value <> 100 then panic('parameter shadow'); end if;
  end procedure;
  procedure Third();
  begin
    const Value: string := 'local';
    Second(100);
    begin
      const Value: boolean := true;
      First();
      if not Value then panic('block shadow'); end if;
    end;
    if Value <> 'local' then panic('local shadow'); end if;
  end procedure;
begin
  Third();
end procedure;
begin
  var Total: integer := 0;
  Outer(var Total);
  if Total <> 2 then panic('capture identity'); end if;
end.
"#,
    );
}

#[test]
fn immutable_sibling_captures_survive_returned_routines_and_anonymous_wrappers() {
    assert_succeeds(
        r#"
program SiblingValues;
function Make(Base: integer): function(): integer;
  function First(Value: integer): integer;
  begin
    return Base + Value;
  end function;
  function Second(): integer;
  begin
    return First(Value := 2);
  end function;
  function Third(): integer;
  begin
    const Invoke: function(): integer := function(): integer begin
      return Second();
    end function;
    return Invoke();
  end function;
begin
  return Third;
end function;
begin
  const Invoke: function(): integer := Make(40);
  if Invoke() <> 42 then panic('transitive value'); end if;
end.
"#,
    );
}

#[test]
fn mutable_sibling_captures_share_one_cell_after_the_parent_returns() {
    assert_succeeds(
        r#"
program SiblingCells;
function Make(): function(): integer;
  function First(): integer;
  begin
    Value := Value + 1;
    return Value;
  end function;
  function Second(): integer;
  begin
    return First();
  end function;
begin
  var Value: integer := 0;
  return Second;
end function;
begin
  const Next: function(): integer := Make();
  const Copy: function(): integer := Next;
  if (Next() <> 1) or (Copy() <> 2) then panic('shared cell'); end if;
end.
"#,
    );
}

#[test]
fn same_named_captures_from_different_ancestors_keep_their_lexical_bindings() {
    assert_succeeds(
        r#"
program AncestorCaptureIdentity;
function Outer(Value: string): function(): integer;
  function First(): string;
  begin
    return Value;
  end function;
  function Inner(Value: integer): function(): integer;
    function Second(): string;
    begin
      return First();
    end function;
    function Third(): integer;
    begin
      const Copy: integer := Value;
      if Second() <> 'outer' then panic('outer capture'); end if;
      return Copy;
    end function;
  begin
    return Third;
  end function;
begin
  return Inner(42);
end function;
begin
  const Invoke: function(): integer := Outer('outer');
  if Invoke() <> 42 then panic('inner capture'); end if;
end.
"#,
    );
}
