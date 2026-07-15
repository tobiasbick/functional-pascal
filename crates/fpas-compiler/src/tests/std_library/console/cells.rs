use super::*;

#[test]
fn std_console_cell_frames_colors_and_bulk_round_trip() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var Kind: ColorKind := Std.Console.ColorKind.Rgb;
  var Fg: Color := RgbColor(10, 20, 30);
  var Bg: Color := Ansi256Color(42);
  var Value: Cell := record glyph := 'X'; foreground := Fg; background := Bg; end;
  BeginFrame();
  FillRect(record x := 1; y := 1; width := 3; height := 1; end,
    record glyph := '.'; foreground := CrtColor(7); background := CrtColor(0); end);
  WriteCells(2, 1, [Value]);
  Present();
  var Actual: Cell := Unwrap(GetCell(2, 1));
  var ActualFg: Color := Actual.foreground;
  var ActualBg: Color := Actual.background;
  WriteLn(Actual.glyph);
  WriteLn(ActualFg.red);
  WriteLn(ActualFg.green);
  WriteLn(ActualFg.blue);
  WriteLn(ActualBg.index);
  WriteLn(Kind = ActualFg.kind);
  WriteLn(DisplayWidth('A中B'))
end.",
    );
    assert_eq!(out.lines, vec!["X", "10", "20", "30", "42", "true", "4"]);
}

#[test]
fn std_console_saved_region_restores_once() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Option;
begin
  var A: Cell := record glyph := 'A'; foreground := CrtColor(7); background := CrtColor(0); end;
  var B: Cell := record glyph := 'B'; foreground := CrtColor(7); background := CrtColor(0); end;
  PutCell(1, 1, A);
  var Saved: SavedRegion := SaveRegion(record x := 1; y := 1; width := 1; height := 1; end);
  PutCell(1, 1, B);
  RestoreRegion(Saved);
  var Actual: Cell := Unwrap(GetCell(1, 1));
  WriteLn(Actual.glyph)
end.",
    );
    assert_eq!(out.lines, vec!["A"]);
}

#[test]
fn std_console_color_constructors_validate_ranges() {
    let message = compile_run_err(
        "\
program T;
uses Std.Console;
begin
  var Invalid: Color := RgbColor(256, 0, 0)
end.",
    );
    assert!(message.contains("0..=255"), "{message}");
}
