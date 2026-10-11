//! Runtime checks, control-flow reachability, and integer representation.

use super::{assert_succeeds, run_program};

#[test]
fn subrange_conversions_preserve_written_evaluation_and_nested_control_flow() {
    assert_succeeds("program T; type Percent = 0..100; type Small = 0..10; type Same = 0..100;
        var Calls: integer := 0;
        function Next(Value: integer): integer; begin Calls := Calls + 1; return Value; end function;
        procedure Set(var Value: Percent); begin Value := Percent(10); end procedure;
        function Make(): function(): integer; begin const Saved := Percent(Next(9));
            return function(): integer begin return Saved + 1; end function; end function;
        begin const Low := Percent(Next(0)); const High := Percent(Next(100));
        var P := Same(5); Set(var P);
        const Half: real := High / Percent(2);
        const Read: function(): integer := Make();
        const Values := [Percent(Next(1)), Percent(if true then Next(2) else 101 end if)];
        var Integers: array of integer := [0, 1]; Integers[Small(1)] := P;
        const Lookup: dict of integer to string := [1: 'one']; const Text: string := 'ab';
        if Integers[Small(1)] <> 10 or Text[Small(1)] <> 'b' or Lookup[Small(1)] <> 'one'
            or not (Small(1) in Lookup) or not (Small(10) in Integers) then panic('range indexing'); end if;
        if false then const Bad := Percent(Next(101)); end if;
        if Calls <> 5 or Low <> 0 or High <> 100 or P <> 10 or Half <> 50.0 or Read() <> 10 or Values[1] <> 2 then panic('range runtime'); end if;
        for I: integer := 0 to 10 do const Current := Small(I); end for;
        case P of when 0..9: panic('case'); when Percent(10): null; else panic('case else'); end case;
        case Some(P) of when Some(Percent(10)): null; when Some(_): panic('nested label'); when None: panic('nested option'); end case;
        end.");
}

#[test]
fn dynamic_subrange_violations_panic_with_value_and_both_bounds() {
    for value in [-1, 101] {
        let error = run_program(&format!("program T; type Percent = 0..100; function Read(): integer; begin return {value}; end function; begin const P := Percent(Read()); end.")).expect_err("range must panic");
        let message = error.message;
        assert!(
            message.contains(&format!("value {value}")) && message.contains("0..100"),
            "{message}"
        );
    }
}

#[test]
fn subrange_membership_checks_inclusive_bounds_without_conversion_or_overflow() {
    assert_succeeds("program T; type Percent = 0..100; type Alias = Percent;
        type Single = -1..-1; const Minimum: integer := -9223372036854775807 - 1;
        type Full = Minimum..9223372036854775807;
        type Bottom = Minimum..Minimum; type Top = 9223372036854775807..9223372036854775807;
        function Read(Value: integer): integer; begin return Value; end function;
        begin
        if not (Read(0) in Percent) or not (Read(100) in Alias)
            or Read(-1) in Percent or Read(101) in Percent then panic('endpoints'); end if;
        if not (Read(-1) in Single) or Read(-2) in Single or Read(0) in Single then panic('singleton'); end if;
        if not (Read(Minimum) in Full) or not (Read(9223372036854775807) in Full)
            or not (Read(Minimum) in Bottom) or Read(Minimum + 1) in Bottom
            or not (Read(9223372036854775807) in Top) or Read(9223372036854775806) in Top
            then panic('integer limits'); end if;
        if not (Percent(50) in Percent) or Percent(50) in Single then panic('subrange input'); end if;
        end.");
}

#[test]
fn subrange_membership_evaluates_once_and_preserves_short_circuit_guards() {
    assert_succeeds("program T; type Percent = 0..100; var Calls: integer := 0;
        function Next(Value: integer): integer; begin Calls := Calls + 1; return Value; end function;
        function Reject(): boolean; begin panic('skipped operand'); end function;
        function Guard(Input: integer): boolean; begin
            return Input in Percent and Percent(Input) >= 0; end function;
        function Make(Input: integer): function(): boolean; begin
            return function(): boolean begin return Input in Percent; end function; end function;
        begin
        const Accepted: boolean := (if true then Next(50) else Next(101) end if) in Percent;
        const Rejected: boolean := Next(101) in Percent and Reject();
        const Skipped: boolean := false and Next(50) in Percent;
        const Negated: boolean := not Next(-1) in Percent;
        const Either: boolean := true or Next(50) in Percent;
        if false then const Unreached: boolean := Next(50) in Percent; end if;
        const Check: function(): boolean := Make(101);
        if Calls <> 3 or not Accepted or Rejected or Skipped or not Negated or not Either
            or not Guard(50) or Guard(101) or Check() then panic('membership evaluation'); end if;
        end.");
}
