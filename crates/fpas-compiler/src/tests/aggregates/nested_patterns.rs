//! Execute recursive patterns with safe payload reads and one selected guard.

use super::assert_succeeds;

#[test]
fn nested_generic_enum_option_result_patterns_bind_concrete_values() {
    assert_succeeds(
        "program Main;\n        type Choice of (T) = enum Present(Value: T); Missing; end enum;\n        type Chain of (T) = enum More(Value: T; Tail: Chain of (T)); Finished; end enum;\n        var Item: Choice of (Result of (Option of (integer), string)) := Choice.Present(Result.Ok(Option.Some(42)));\n        var Root: Chain of (integer) := Chain.More(1, Chain.More(42, Chain.Finished));\n        begin\n          case Item of\n            when Choice.Present(Result.Ok(Option.Some(const Value))): if Value <> 42 then panic('nested binding'); end if;\n            when Choice.Present(Result.Ok(Option.None)): panic('none');\n            when Choice.Present(Result.Error(_)): panic('error');\n            when Choice.Missing: panic('missing');\n          end case;\n          case Root of\n            when Chain.More(_, Chain.More(const Value, _)): if Value <> 42 then panic('recursive binding'); end if;\n            when Chain.More(_, Chain.Finished): panic('short chain');\n            when Chain.Finished: panic('empty chain');\n          end case;\n        end program;",
    );
}

#[test]
fn grouped_matching_patterns_run_a_false_guard_once_then_continue_to_next_arm() {
    assert_succeeds("program Main;
        type Pair = enum Both(A: boolean; B: boolean); end enum;
        mutable var GuardCalls: integer := 0;
        mutable var ScrutineeCalls: integer := 0;
        function ReadPair(): Pair; begin ScrutineeCalls := ScrutineeCalls + 1; return Pair.Both(true, true); end function;
        function Guard(): boolean; begin GuardCalls := GuardCalls + 1; return false; end function;
        begin
          case ReadPair() of
            when Pair.Both(true, _), Pair.Both(_, true) if Guard(): panic('false guard');
            when Pair.Both(_, _): null;
          end case;
          if (GuardCalls <> 1) or (ScrutineeCalls <> 1) then panic('evaluation count'); end if;
        end program;");
}

#[test]
fn failed_outer_patterns_do_not_read_inactive_payloads() {
    assert_succeeds(
        "program Main;
        type Choice = enum Present(Value: Option of (integer)); Missing; end enum;
        var Item: Choice := Choice.Missing;
        begin case Item of
          when Choice.Present(Option.Some(const Value)): panic('present');
          when Choice.Present(Option.None): panic('none');
          when Choice.Missing: null;
        end case; end program;",
    );
}

#[test]
fn pattern_bound_callables_can_escape_in_a_closure() {
    assert_succeeds("program Main;
        type Choice of (T) = enum Present(Value: T); Missing; end enum;
        function Make(Item: Choice of (function(Value: integer): integer)): function(): integer;
        begin case Item of
          when Choice.Present(const Action): return function(): integer begin return Action(41); end function;
          when Choice.Missing: return function(): integer begin return 0; end function;
        end case; end function;
        begin var Action: function(): integer := Make(Choice.Present(function(Value: integer): integer begin return Value + 1; end function));
          if Action() <> 42 then panic('captured pattern callable'); end if;
        end program;");
}

#[test]
fn grouped_payload_bindings_share_one_owner_when_captured_by_an_escaping_closure() {
    assert_succeeds("program Main;
        type Pair = enum Both(A: integer; B: integer); end enum;
        function Make(Item: Pair): function(): integer;
        begin case Item of
          when Pair.Both(0, const Value), Pair.Both(const Value, 0): return function(): integer begin return Value; end function;
          when Pair.Both(const Left, const Right): return function(): integer begin return Left + Right; end function;
        end case; end function;
        begin var Action: function(): integer := Make(Pair.Both(42, 0));
          var Other: function(): integer := Make(Pair.Both(20, 22));
          if (Action() <> 42) or (Other() <> 42) then panic('grouped binding capture'); end if;
        end program;");
}
