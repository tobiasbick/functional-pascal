//! Counting-loop termination at inclusive integer bounds.

use super::super::assert_succeeds;

#[test]
fn counting_loop_bounds_use_the_outer_binding() {
    assert_succeeds(
        "program OuterBounds; begin
          var I: integer := 3;
          mutable var Count: integer := 0;
          for I: integer := 1 to I do Count := Count + 1;
          if Count <> 3 then panic('ascending end used the inner binding');
          var J: integer := 1;
          for J: integer := 3 downto J do Count := Count + 1;
          if Count <> 6 then panic('descending end used the inner binding');
          if (I <> 3) or (J <> 1) then panic('outer bindings were changed')
        end.",
    );
}

#[test]
fn nested_counting_loop_bounds_use_the_enclosing_counter() {
    assert_succeeds(
        "program NestedBounds; begin
          mutable var Count: integer := 0;
          for I: integer := 1 to 3 do
          begin
            for I: integer := 1 to I do Count := Count + 1;
            Count := Count + I
          end;
          if Count <> 12 then panic('nested bound or outer counter changed')
        end.",
    );
}

#[test]
fn counting_loop_bounds_are_evaluated_once_in_source_order() {
    assert_succeeds(
        "program BoundOrder;
        mutable var Trace: integer := 0;
        function Start(): integer;
        begin
          Trace := Trace * 10 + 1;
          return 1
        end;
        function Finish(): integer;
        begin
          Trace := Trace * 10 + 2;
          return 3
        end;
        begin
          mutable var Count: integer := 0;
          for I: integer := Start() to Finish() do Count := Count + 1;
          if (Trace <> 12) or (Count <> 3) then panic('bounds evaluated out of order or repeatedly')
        end.",
    );
}

#[test]
fn counting_loops_stop_at_integer_extremes() {
    for (start, direction, end, expected) in [
        ("9223372036854775807", "to", "9223372036854775807", 1),
        ("9223372036854775806", "to", "9223372036854775807", 2),
        (
            "(-9223372036854775807 - 1)",
            "downto",
            "(-9223372036854775807 - 1)",
            1,
        ),
        (
            "-9223372036854775807",
            "downto",
            "(-9223372036854775807 - 1)",
            2,
        ),
    ] {
        for tail in ["", "; continue"] {
            assert_succeeds(&format!(
                "program CountingBounds; begin
                  mutable var Count: integer := 0;
                  for I: integer := {start} {direction} {end} do
                  begin
                    Count := Count + 1;
                    if Count > {expected} then panic('counter wrapped'){tail}
                  end;
                  if Count <> {expected} then panic('wrong iteration count')
                end."
            ));
        }
    }
}

#[test]
fn counting_loops_skip_empty_ranges() {
    assert_succeeds(
        "program EmptyRanges; begin
          for I: integer := 9223372036854775807 to 9223372036854775806 do
            panic('ascending empty range');
          for I: integer := (-9223372036854775807 - 1) downto -9223372036854775807 do
            panic('descending empty range')
        end.",
    );
}

#[test]
fn counting_loops_break_before_the_next_iteration() {
    assert_succeeds(
        "program BreakBounds; begin
          mutable var Count: integer := 0;
          for I: integer := 9223372036854775806 to 9223372036854775807 do
          begin
            Count := Count + 1;
            break
          end;
          for I: integer := -9223372036854775807 downto (-9223372036854775807 - 1) do
          begin
            Count := Count + 1;
            break
          end;
          if Count <> 2 then panic('break did not leave the loop')
        end.",
    );
}
