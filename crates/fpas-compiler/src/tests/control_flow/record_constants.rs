//! Record-derived constant labels retain their values in compiled pattern dispatch.
//! See `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::assert_succeeds;

#[test]
fn record_constant_patterns_dispatch_boolean_and_enum_values() {
    assert_succeeds("program RecordPatterns;
        type Color = enum Red; Green; end enum;
        type Flags = record Enabled: boolean := true; Color: Color := Color.Red; end record;
        type Settings = record Flags: Flags; end record;
        const Config: Settings := Settings(Flags := Flags( ));
        const Updated: Flags := Config.Flags with Enabled := false; Color := Color.Green; end with;
        const Enabled: boolean := Config.Flags.Enabled;
        const Selected: Color := Updated.Color;
        function BooleanScore(Value: Option of boolean): integer;
        begin case Value of when Some(Enabled): return 1;
        when Some(Updated.Enabled): return 2; when None: return 4; end case; end function;
        function ColorScore(Value: result of Option of Color, string): integer;
        begin case Value of when Ok(Some(Config.Flags.Color)): return 8;
        when Ok(Some(Selected)): return 16; when Ok(None): return 32;
        when Error(_): return 64; end case; end function;
        begin
        if BooleanScore(Some(true)) + BooleanScore(Some(false)) + BooleanScore(None) <> 7 then panic('boolean dispatch'); end if;
        if ColorScore(Ok(Some(Color.Red))) + ColorScore(Ok(Some(Color.Green))) + ColorScore(Ok(None)) + ColorScore(Error('x')) <> 120 then panic('enum dispatch'); end if;
        const Candidate: Option of boolean := Some(true);
        if Candidate is Some(Config.Flags.Enabled) then null; else panic('is projection'); end if;
        end.");
}
