//! Inferred local storage, conversions, static labels, and captured mutable cells.

use super::assert_succeeds;

#[test]
fn determined_local_expressions_execute_once_with_fixed_types() {
    assert_succeeds("program T; var Trace: string := '';
        type Point = record X: integer; end record;
        function Next(Tag: string): integer; begin Trace := Trace + Tag; return Trace.Length(); end function;
        function Make(): Point; begin return Point(X := 7); end function;
        begin const First := Next('A'); const Second := Next('B');
            var P := Make(); const Copy := P; const Field := P.X; const Sum := First + Second;
            const Updated := P with X := Sum; end with;
            const Choose := if First = 1 then Updated else Copy end if;
            const Read := function(Value: integer): integer begin return Value + Field; end function;
            const Called := Read(Sum); const Numbers := [First, Second]; const Indexed := Numbers[1];
            const Label := case Indexed of when 2: 'two'; else 'other'; end case;
            P.X := 9;
            for I: integer := 1 to 2 do const Iteration := Next('C'); end for;
            if false then const Skipped := Next('X'); end if;
            if Trace <> 'ABCC' or Copy.X <> 7 or Choose.X <> 3 or Called <> 10 or Label <> 'two' then panic('determined initializer'); end if;
        end.");
}

#[test]
fn expected_results_complete_generic_calls_in_value_contexts() {
    assert_succeeds("program T;
        function Empty<T>(): Option of T; begin return None; end function;
        function Choose<T>(Value: Option of T; Evidence: T): Option of T; begin return Value; end function;
        function Succeed<T, E>(Value: T): Result of (T, E); begin return Ok(Value); end function;
        function Returned(): Option of integer; begin return Empty(); end function;
        procedure Accept(Value: Option of integer); begin if not Value.IsNone() then panic('argument'); end if; end procedure;
        type Holder = record Value: Option of integer; end record;
        begin const Direct: Option of integer := Empty();
            var Assigned: Option of integer := Some(1); Assigned := Empty();
            const Field := Holder(Value := Empty());
            const Selected: Option of integer := Choose(Empty(), 1);
            const Reordered: Option of integer := Choose(Evidence := 1, Value := Empty());
            const Partial: Result of (integer, string) := Succeed(3);
            const Factory: function(): Option of integer := Empty;
            Accept(Empty());
            if not Direct.IsNone() or not Assigned.IsNone() or not Field.Value.IsNone() or not Selected.IsNone() or not Reordered.IsNone() or not Returned().IsNone() or not Factory().IsNone() or Partial.Unwrap() <> 3 then panic('contextual call'); end if;
        end.");
}

#[test]
fn expected_results_complete_static_instance_and_postfix_methods() {
    assert_succeeds("program T;
        type Factory = record
            static function Empty<T>(): Option of T; begin return None; end function;
            function Missing<T>(Self: Factory): Option of T; begin return None; end function;
        end record;
        begin const F := Factory();
            const StaticValue: Option of integer := Factory.Empty();
            const Instance: Option of string := F.Missing();
            const Postfix: Option of integer := Factory().Missing();
            if not StaticValue.IsNone() or not Instance.IsNone() or not Postfix.IsNone() then panic('contextual method'); end if;
        end.");
}

#[test]
fn native_factories_callbacks_and_callable_wrappers_use_result_context() {
    assert_succeeds("program T;
        function Empty<T>(): Option of T; begin return None; end function;
        function Missing<T>(Value: integer): Option of T; begin return None; end function;
        function Identity<T>(Value: T): T; begin return Value; end function;
        function Keep<A>(Value: A): A; begin return Value; end function;
        begin const Values: array of Option of integer := array.Fill(Count := 2, Value := Empty());
            const Mapped: array of Option of string := [1].Map(Missing);
            const Items := [1]; const Direct: array of Option of integer := Items.Map(Missing);
            const F: function(Value: integer): integer := Keep(Identity);
            const Callbacks: array of function(Value: integer): integer := Keep([Identity]);
            if Values.Length() <> 2 or not Values[0].IsNone() or not Mapped[0].IsNone() or not Direct[0].IsNone() or F(3) <> 3 or Callbacks[0](4) <> 4 then panic('native or callable context'); end if;
        end.");
}

#[test]
fn decision_siblings_complete_calls_without_repeating_evaluation() {
    assert_succeeds("program T; var Calls: integer := 0;
        function Empty<T>(): Option of T; begin Calls := Calls + 1; return None; end function;
        begin const Selected := if true then Empty() else Some(1) end if;
            const Outcome := case 1 of when 1: Empty(); else Some(2); end case;
            const Input := Some(4);
            const Bound := case Input of when Some(const Value): if Value > 0 then Empty() else Some(Value) end if; when None: Some(0); end case;
            if not Selected.IsNone() or not Outcome.IsNone() or not Bound.IsNone() or Calls <> 3 then panic('decision context'); end if;
        end.");
}

#[test]
fn inferred_aggregate_and_scalar_storage_runs_with_the_checked_types() {
    assert_succeeds("program T; type UserId = distinct integer; begin
        const Whole := 42; const LowerBound := -9223372036854775807; const Fraction := 1.5;
        const Flag := true; const Text := 'Ada'; const Values := [[1, 2], [3, 4]];
        var Index := ['a': 1, 'b': 2]; Index['a'] := 3;
        const Id := UserId(Whole); const Raw := integer(Id);
        const Options := [None, Some(Whole)]; const Matrix := [[], [Whole]];
        const Outcomes := [Ok(Whole), Error('failed')];
        if not Options[0].IsNone() or Options[1].Unwrap() <> 42 or Matrix[0].Length() <> 0 or Matrix[1][0] <> 42 or Outcomes[0].Unwrap() <> 42 or not Outcomes[1].IsError() then panic('joint collection evidence'); end if;
        if Raw <> 42 or LowerBound >= 0 or Fraction <> 1.5 or not Flag or Text <> 'Ada' or Values[1][0] <> 3 or Index['a'] <> 3 then panic('inferred types'); end if;
        case Raw of when Whole: null; else panic('static label'); end case; end.");
}

#[test]
fn inferred_bindings_keep_copy_captures_cells_and_named_nested_captures() {
    assert_succeeds("program T;
        function Make(): function(): integer;
        function Named(): integer; begin return Saved + Count; end function;
        begin const Saved := 7; var Count := 2;
            const Bump: procedure() := procedure() begin Count := Count + 1; end procedure;
            Bump(); return Named;
        end function;
        begin const Read: function(): integer := Make();
            if Read() <> 10 then panic('named capture'); end if;
            const Saved := 5; const Copy: function(): integer := function(): integer begin return Saved; end function;
            begin const Saved := 99; end;
            if Copy() <> 5 then panic('copy capture'); end if;
        end.");
}

#[test]
fn inferred_conversion_initializers_execute_once_when_reached() {
    assert_succeeds(
        "program T; type UserId = distinct integer; var Calls: integer := 0;
        function Next(): integer; begin Calls := Calls + 1; return Calls; end function;
        begin const First := UserId(Next());
            for I: integer := 1 to 2 do const Id := UserId(Next()); end for;
            if false then const Missing := UserId(Next()); end if;
            if Calls <> 3 or integer(First) <> 1 then panic('initializer order'); end if;
        end.",
    );
}

#[test]
fn inferred_record_cells_and_enum_copy_captures_escape_named_nested_routines() {
    assert_succeeds("program T;
        type Box of T = record Value: T; end record;
        type Lookup of T = enum Found(Value: T); Missing; end enum;
        function Make(): function(): integer;
        function Read(): integer;
        begin case Saved of
            when Lookup.Found(const Value): return Value + State.Value;
            when Lookup.Missing: return State.Value;
        end case; end function;
        begin const Saved := Lookup.Found(7); var State := Box(Value := 2);
            const Bump: procedure() := procedure() begin State.Value := State.Value + 1; end procedure;
            Bump(); return Read;
        end function;
        begin const Read: function(): integer := Make();
            if Read() <> 10 then panic('constructor captures'); end if;
            const Nested := Some(Box(Value := Box(Value := 'nested')));
            case Nested of
                when Some(const Value): if Value.Value.Value <> 'nested' then panic('nested layout'); end if;
                when None: panic('missing layout');
            end case;
        end.");
}

#[test]
fn inferred_constructor_initializers_preserve_argument_default_and_statement_order() {
    assert_succeeds("program T; var Trace: string := '';
        function Next(Tag: string): integer; begin Trace := Trace + Tag; return Trace.Length(); end function;
        type Stamp = record First: integer := Next('D'); Second: integer := Next('E'); end record;
        type Shape = enum Pair(Left: integer; Right: integer); Idle; end enum;
        begin const P := Stamp(Second := Next('S'));
            const Q := Shape.Pair(Right := Next('R'), Left := Next('L'));
            if Trace <> 'SDRL' or P.First <> 2 or P.Second <> 1 then panic('construction order'); end if;
            case Q of when Shape.Pair(const Left, const Right):
                if Left <> 4 or Right <> 3 then panic('payload order'); end if;
                when Shape.Idle: panic('wrong variant'); end case;
            for I: integer := 1 to 2 do const Defaulted := Stamp(); end for;
            if false then const Missing := Stamp(); end if;
            if Trace <> 'SDRLDEDE' then panic('statement order'); end if;
        end.");
}
