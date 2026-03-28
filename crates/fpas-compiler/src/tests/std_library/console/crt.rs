use super::*;

#[test]
fn std_console_crt_wind_min_max_follow_window() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  Window(10, 5, 20, 8);
  WriteLn(WindMin());
  WriteLn(WindMax())
end.",
    );
    assert_eq!(out.lines, vec!["1290", "2068"]);
}

#[test]
fn std_console_crt_window_cursor_and_clear() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  Window(10, 5, 20, 8);
  var A: integer := WhereX();
  var B: integer := WhereY();
  GotoXY(3, 2);
  var C: integer := WhereX();
  var D: integer := WhereY();
  ClrScr();
  var E: integer := WhereX();
  var F: integer := WhereY();
  WriteLn(A);
  WriteLn(B);
  WriteLn(C);
  WriteLn(D);
  WriteLn(E);
  WriteLn(F)
end.",
    );
    assert_eq!(out.lines, vec!["1", "1", "3", "2", "1", "1"]);
}

#[test]
fn std_console_crt_color_constants_and_delay() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  TextColor(LightRed);
  TextBackground(Blue);
  Delay(0);
  WriteLn(LightRed);
  WriteLn(Blue)
end.",
    );
    assert_eq!(out.lines, vec!["12", "1"]);
}

#[test]
fn std_console_crt_text_attr_modes_and_compat_calls() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;
begin
  SetTextAttr(30);
  WriteLn(TextAttr());
  LowVideo();
  WriteLn(TextAttr());
  HighVideo();
  WriteLn(TextAttr());
  NormVideo();
  WriteLn(TextAttr());
  TextMode(C80);
  WriteLn(LastMode());
  WriteLn(ScreenWidth());
  WriteLn(ScreenHeight());
  CursorBig();
  DelLine();
  InsLine();
  Sound(440);
  NoSound();
  AssignCrt()
end.",
    );
    assert_eq!(out.lines, vec!["30", "22", "30", "7", "3", "80", "25"]);
}
