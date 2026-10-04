//! Executable generic record construction and source evaluation order.

use super::assert_succeeds;

#[test]
fn generic_record_constructors_keep_concrete_field_and_callable_values() {
    assert_succeeds(
        r#"program Main;
        type Box of (T) = record Value: T; Count: integer := 3; end record;
        function ReadBox of (U)(Item: Box of (U)): U; begin return Item.Value; end function;
        function Make of (U)(Item: U): Box of (U); begin return Box(Value := Item); end function;
        const Number: Box of (integer) := Make(42);
        const Action: Box of (function(Value: integer): integer) := Box(Value := function(Value: integer): integer begin return Value + 1; end function);
        begin
            if ReadBox(Number) <> 42 then panic('generic integer'); end if;
            if (Action.Value(41) <> 42) or (Action.Count <> 3) then panic('callable field'); end if;
        end program;"#,
    );
}

#[test]
fn nested_and_recursive_generic_values_use_one_finite_layout() {
    assert_succeeds(
        r#"program Main;
        type Node of (T) = record Value: T; Children: array of (Node of (T)); end record;
        const Leaf: Node of (integer) := Node(Value := 42, Children := []);
        const Root: Node of (integer) := Node(Children := [Leaf], Value := 1);
        begin if Root.Children[0].Value <> 42 then panic('recursive field'); end if; end program;"#,
    );
}

#[test]
fn nested_generic_paths_support_copy_updates_and_callable_replacement() {
    assert_succeeds(
        r#"program Main;
        type Box of (T) = record Value: T; end record;
        type Item = record Count: integer; end record;
         var Items: Box of (array of (Item)) := Box(Value := [Item(Count := 1)]);
         var Action: Box of (function(Value: integer): integer) := Box(Value := function(Value: integer): integer begin return Value; end function);
        begin
            Items.Value[0].Count := 42;
            Action.Value := function(Value: integer): integer begin return Value + 1; end function;
            const Changed: Box of (array of (Item)) := Items with Value := []; end with;
            if (Items.Value[0].Count <> 42) or (Action.Value(41) <> 42) or (Changed.Value <> []) then panic('generic update'); end if;
        end program;"#,
    );
}

#[test]
fn supplied_fields_run_in_written_order_before_omitted_defaults() {
    assert_succeeds(
        r#"program Main;
         var Trace: integer := 0;
        function Mark(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
        type Settings = record First: integer; Second: integer; Third: integer := 3; Fourth: integer := 4; end record;
        begin
            const Item: Settings := Settings(Second := Mark(2), First := Mark(1));
            if (Trace <> 21) or (Item.First <> 1) or (Item.Second <> 2) or (Item.Third <> 3) or (Item.Fourth <> 4) then panic('field evaluation order'); end if;
        end program;"#,
    );
}

#[test]
fn empty_constructor_defaults_keep_closure_and_alias_identities() {
    assert_succeeds(
        r#"program Main;
        type Actions = record Next: function(Value: integer): integer := function(Value: integer): integer begin return Value + 1; end function; end record;
        type Alias = Actions;
        const Item: Alias := Alias();
        begin if Item.Next(41) <> 42 then panic('alias closure default'); end if; end program;"#,
    );
}

#[test]
fn generic_enum_construction_keeps_nominal_values_and_payload_order() {
    assert_succeeds(
        r#"program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        type Pair of (T) = enum Values(First: T; Second: T); Missing; end enum;
         var Trace: integer := 0;
        function Mark(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
        function Make of (U)(Value: U): Choice of (U); begin return Choice.Present(Value); end function;
        const Number: Choice of (integer) := Make(42);
        const Text: Choice of (string) := Choice.Present('value');
        const MissingNumber: Choice of (integer) := Choice.Missing;
        begin
            const Ordered: Pair of (integer) := Pair.Values(Mark(1), Mark(2));
            if (Number <> Choice.Present(42)) or (Text <> Choice.Present('value')) or (MissingNumber = Number) or (Trace <> 12) then panic('generic enum'); end if;
        end program;"#,
    );
}

#[test]
fn generic_enum_patterns_recover_concrete_payload_types() {
    assert_succeeds(
        r#"program Main;

type Choice of (T) = enum
  Present(Value: T);
  Missing;
end enum;

const Item: Choice of (function(Value: integer): integer) := Choice.Present(function(Value: integer): integer begin
  return Value + 1;
end function);

begin
  case Item of
    when Choice.Present(const Action):
      if Action(41) <> 42 then
        panic('generic payload');
      end if;
    when Choice.Missing:
      panic('unexpected variant');
  end case;
end program;
"#,
    );
}
