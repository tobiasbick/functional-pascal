//! Counting-loop ordinal types, direction, and empty Boolean ranges.

use super::super::assert_succeeds;

#[test]
fn boolean_counting_loops_preserve_values_in_both_directions() {
    for (start, direction, end, expected) in [
        (false, "to", false, 1),
        (false, "to", true, 12),
        (true, "to", false, 0),
        (true, "to", true, 2),
        (false, "downto", false, 1),
        (false, "downto", true, 0),
        (true, "downto", false, 21),
        (true, "downto", true, 2),
    ] {
        for tail in ["", "; continue"] {
            assert_succeeds(&format!(
                "program BooleanBounds; begin
                  mutable var Count: integer := 0;
                  mutable var Values: integer := 0;
                  for B: boolean := {start} {direction} {end} do
                  begin
                    Count := Count + 1;
                    if Count > 2 then panic('boolean counter did not stop');
                    if B then Values := Values * 10 + 2
                    else Values := Values * 10 + 1{tail}
                  end;
                  if Values <> {expected} then panic('wrong boolean values or direction')
                end."
            ));
        }
    }
}

#[test]
fn boolean_counting_bound_can_shadow_an_outer_boolean() {
    assert_succeeds(
        "program BooleanShadow; begin
          var B: boolean := true;
          mutable var Count: integer := 0;
          for B: boolean := false to B do Count := Count + 1;
          if (Count <> 2) or not B then panic('boolean bound or outer value changed')
        end.",
    );
}

#[test]
fn simple_enum_counting_loops_keep_their_ordinal_values() {
    assert_succeeds(
        "program EnumBounds;
        type Color = enum Red; Green; Blue; end;
        begin
          mutable var Count: integer := 0;
          mutable var Greens: integer := 0;
          for C: Color := Color.Red to Color.Blue do
          begin
            Count := Count + 1;
            if C = Color.Green then Greens := Greens + 1
          end;
          for C: Color := Color.Blue downto Color.Red do
          begin
            Count := Count + 1;
            if C = Color.Green then Greens := Greens + 1
          end;
          if (Count <> 6) or (Greens <> 2) then panic('enum values changed')
        end.",
    );
}
