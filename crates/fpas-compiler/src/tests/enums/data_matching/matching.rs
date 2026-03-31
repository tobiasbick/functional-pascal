use super::*;

#[test]
fn enum_data_case_multiple_labels_keep_each_label_pattern() {
    let out = compile_and_run(
        "\
program EnumMultiLabelPattern;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Square(Side: real);
  Triangle(A: real; B: real; C: real);
end;
begin
  var S: Shape := Shape.Square(5.0);
  case S of
    Shape.Circle(R),
    Shape.Square(R):
      WriteLn('round or square')
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["round or square"]);
}

#[test]
fn enum_data_else_branch() {
    let out = compile_and_run(
        "\
program EnumElse;
uses Std.Console;
type Shape = enum
  Circle(Radius: real);
  Rectangle(Width: real; Height: real);
  Triangle(A: real; B: real; C: real);
end;
begin
  var S: Shape := Shape.Triangle(3.0, 4.0, 5.0);
  case S of
    Shape.Circle(R): WriteLn('circle');
    Shape.Rectangle(W, H): WriteLn('rectangle')
  else
    WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

#[test]
fn enum_data_fieldless_variant_matched_in_data_enum() {
    let out = compile_and_run(
        "\
program EnumFieldless;
uses Std.Console;
type Token = enum
  Eof;
  Number(Value: integer);
  Word(Text: string);
end;
begin
  var T1: Token := Token.Eof;
  var T2: Token := Token.Number(42);
  var T3: Token := Token.Word('hello');
  case T1 of
    Token.Eof: WriteLn('eof');
    Token.Number(V): WriteLn(V);
    Token.Word(S): WriteLn(S)
  end;
  case T2 of
    Token.Eof: WriteLn('eof');
    Token.Number(V): WriteLn(V);
    Token.Word(S): WriteLn(S)
  end;
  case T3 of
    Token.Eof: WriteLn('eof');
    Token.Number(V): WriteLn(V);
    Token.Word(S): WriteLn(S)
  end
end.",
    );
    assert_eq!(out.lines, vec!["eof", "42", "hello"]);
}

#[test]
fn enum_data_all_variants_have_fields() {
    let out = compile_and_run(
        "\
program EnumAllFields;
uses Std.Console, Std.Conv;
type Op = enum
  Add(A: integer; B: integer);
  Mul(A: integer; B: integer);
  Neg(A: integer);
end;
begin
  var O1: Op := Op.Add(10, 20);
  var O2: Op := Op.Mul(3, 7);
  var O3: Op := Op.Neg(5);
  case O1 of
    Op.Add(A, B): WriteLn(Std.Conv.IntToStr(A + B));
    Op.Mul(A, B): WriteLn(Std.Conv.IntToStr(A * B));
    Op.Neg(A): WriteLn(Std.Conv.IntToStr(0 - A))
  end;
  case O2 of
    Op.Add(A, B): WriteLn(Std.Conv.IntToStr(A + B));
    Op.Mul(A, B): WriteLn(Std.Conv.IntToStr(A * B));
    Op.Neg(A): WriteLn(Std.Conv.IntToStr(0 - A))
  end;
  case O3 of
    Op.Add(A, B): WriteLn(Std.Conv.IntToStr(A + B));
    Op.Mul(A, B): WriteLn(Std.Conv.IntToStr(A * B));
    Op.Neg(A): WriteLn(Std.Conv.IntToStr(0 - A))
  end
end.",
    );
    assert_eq!(out.lines, vec!["30", "21", "-5"]);
}
