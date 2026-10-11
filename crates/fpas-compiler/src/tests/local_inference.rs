//! Inferred local storage, conversions, static labels, and captured mutable cells.

use super::assert_succeeds;

#[test]
fn inferred_aggregate_and_scalar_storage_runs_with_the_checked_types() {
    assert_succeeds("program T; type UserId = distinct integer; begin
        const Whole := 42; const LowerBound := -9223372036854775807; const Fraction := 1.5;
        const Flag := true; const Text := 'Ada'; const Values := [[1, 2], [3, 4]];
        var Index := ['a': 1, 'b': 2]; Index['a'] := 3;
        const Id := UserId(Whole); const Raw := integer(Id);
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
