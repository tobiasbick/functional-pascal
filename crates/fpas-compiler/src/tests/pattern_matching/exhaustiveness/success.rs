use super::super::*;

#[test]
fn exhaustiveness_ok_all_enum_variants_covered() {
    let out = compile_and_run(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Yellow;
  case L of
    Light.Red: Std.Console.WriteLn('stop');
    Light.Yellow: Std.Console.WriteLn('caution');
    Light.Green: Std.Console.WriteLn('go')
  end
end.",
    );
    assert_eq!(out.lines, vec!["caution"]);
}

#[test]
fn exhaustiveness_ok_else_branch_present() {
    let out = compile_and_run(
        "\
program T;
type Light = enum Red; Yellow; Green; end;
begin
  var L: Light := Light.Yellow;
  case L of
    Light.Red: Std.Console.WriteLn('stop')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

#[test]
fn exhaustiveness_ok_result_both_covered() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Error('oops');
  case R of
    Ok(V): Std.Console.WriteLn(Std.Conv.IntToStr(V));
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["oops"]);
}

#[test]
fn exhaustiveness_ok_result_with_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Error('oops');
  case R of
    Ok(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  else
    Std.Console.WriteLn('fallback')
  end
end.",
    );
    assert_eq!(out.lines, vec!["fallback"]);
}

#[test]
fn exhaustiveness_ok_option_both_covered() {
    let out = compile_and_run(
        "\
program T;
begin
  var O: Option of integer := None;
  case O of
    Some(V): Std.Console.WriteLn(Std.Conv.IntToStr(V));
    None: Std.Console.WriteLn('absent')
  end
end.",
    );
    assert_eq!(out.lines, vec!["absent"]);
}

#[test]
fn exhaustiveness_ok_option_with_else() {
    let out = compile_and_run(
        "\
program T;
begin
  var O: Option of integer := None;
  case O of
    Some(V): Std.Console.WriteLn(Std.Conv.IntToStr(V))
  else
    Std.Console.WriteLn('fallback')
  end
end.",
    );
    assert_eq!(out.lines, vec!["fallback"]);
}

#[test]
fn exhaustiveness_ok_data_enum_all_covered() {
    let out = compile_and_run(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Shape.Circle(R): Std.Console.WriteLn('circle');
    Shape.Rectangle(W, H): Std.Console.WriteLn('rect');
    Shape.Point: Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["point"]);
}

#[test]
fn exhaustiveness_ok_data_enum_with_else() {
    let out = compile_and_run(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Rectangle(Width: real; Height: real);
    Point;
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Shape.Circle(R): Std.Console.WriteLn('circle')
  else
    Std.Console.WriteLn('other')
  end
end.",
    );
    assert_eq!(out.lines, vec!["other"]);
}

#[test]
fn exhaustiveness_not_required_on_integer() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 99;
  case X of
    1: Std.Console.WriteLn('one')
  end
end.",
    );
    assert!(out.lines.is_empty());
}
