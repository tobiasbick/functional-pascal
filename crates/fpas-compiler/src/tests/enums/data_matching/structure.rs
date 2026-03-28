use super::*;

#[test]
fn enum_data_three_field_variant() {
    let out = compile_and_run(
        "\
program EnumThree;
uses Std.Console, Std.Conv;
type Color = enum
  Rgb(R: integer; G: integer; B: integer);
  Gray(Level: integer);
end;
begin
  var C: Color := Color.Rgb(255, 128, 0);
  case C of
    Color.Rgb(R, G, B): WriteLn(Std.Conv.IntToStr(R) + ',' + Std.Conv.IntToStr(G) + ',' + Std.Conv.IntToStr(B));
    Color.Gray(L): WriteLn(Std.Conv.IntToStr(L))
  end
end.",
    );
    assert_eq!(out.lines, vec!["255,128,0"]);
}

#[test]
fn enum_data_case_with_begin_end_block() {
    let out = compile_and_run(
        "\
program EnumBlock;
uses Std.Console, Std.Conv;
type Wrap = enum
  Pair(X: integer; Y: integer);
end;
begin
  var P: Wrap := Wrap.Pair(10, 20);
  case P of
    Wrap.Pair(X, Y): begin
      var Sum: integer := X + Y;
      var Diff: integer := X - Y;
      WriteLn(Std.Conv.IntToStr(Sum));
      WriteLn(Std.Conv.IntToStr(Diff))
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["30", "-10"]);
}
