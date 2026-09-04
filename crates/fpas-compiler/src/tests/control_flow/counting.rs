//! Counting-loop termination at inclusive integer bounds.

use super::super::assert_succeeds;

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
