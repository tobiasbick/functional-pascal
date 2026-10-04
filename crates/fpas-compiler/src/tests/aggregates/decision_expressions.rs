//! Lazy selection, nested patterns, constructor context, and expression continuations.

use super::assert_succeeds;

#[test]
fn if_values_are_lazy_and_conditions_and_selected_branches_run_in_written_order() {
    assert_succeeds("program Main;
        mutable var Trace: integer := 0;
        function Condition(Value: integer; Answer: boolean): boolean; begin Trace := Trace * 10 + Value; return Answer; end function;
        function Branch(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
        begin var Value: integer := if Condition(1, false) then Branch(9) elsif Condition(2, true) then Branch(3) else Branch(8) end if;
          if (Value <> 3) or (Trace <> 123) then panic('lazy if'); end if;
        end program;");
}

#[test]
fn case_values_use_nested_bindings_and_exactly_one_scrutinee_and_selected_expression() {
    assert_succeeds(
        "program Main;\n        type Choice of (T) = enum Present(Value: T); Missing; end enum;\n        mutable var Calls: integer := 0;\n        function Make(): Choice of (Option of (integer)); begin Calls := Calls + 1; return Choice.Present(Option.Some(42)); end function;\n        begin var Value: integer := 1 + case Make() of\n          when Choice.Present(Option.Some(const Number)) if Number < 0: 0;\n          when Choice.Present(Option.Some(const Number)): Number;\n          when Choice.Present(Option.None): 0;\n          when Choice.Missing: 0;\n        end case;\n          if (Value <> 43) or (Calls <> 1) then panic('lazy case'); end if;\n        end program;",
    );
}

#[test]
fn selected_callables_and_constructor_values_remain_typed_across_merge_blocks() {
    assert_succeeds("program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        type Box of (T) = record Value: T; end record;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        begin
          var Item: Choice of (integer) := Identity(if false then Choice.Missing else Choice.Present(42) end if);
          var Value: Box of (integer) := Box(Value := if true then 42 else 0 end if);
          var ArrayValue: array of (integer) := if false then [] else [42] end if;
          var Action: function(): integer := if true then function(): integer begin return Value.Value; end function else function(): integer begin return 0; end function end if;
          if (Action() <> 42) or (ArrayValue[0] <> 42) then panic('merged types'); end if;
          var Chosen: function(): integer := case Item of
            when Choice.Present(const Number): function(): integer begin return Number; end function;
            when Choice.Missing: function(): integer begin return 0; end function;
          end case;
          if Chosen() <> 42 then panic('case closure'); end if;
        end program;");
}

#[test]
fn selected_procedure_values_and_nested_continuations_preserve_evaluation_order() {
    assert_succeeds("program Main;
        mutable var Trace: integer := 0;
        procedure First(); begin Trace := Trace * 10 + 1; end procedure;
        procedure Second(); begin Trace := Trace * 10 + 2; end procedure;
        function Mark(Value: integer): integer; begin Trace := Trace * 10 + Value; return Value; end function;
        function Combine(Left: integer; Right: integer): integer; begin return Left + Right; end function;
        begin
          var Action: procedure() := if false then First else Second end if;
          Action();
          var Value: integer := Combine(Mark(3), if true then case false of when true: Mark(9); when false: Mark(4); end case else Mark(8) end if);
          if (Trace <> 234) or (Value <> 7) then panic('selected procedures and arguments'); end if;
        end program;");
}
