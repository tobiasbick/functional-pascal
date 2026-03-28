use super::*;

// ===========================================================================
// Guard clauses — enum with data
// ===========================================================================

#[test]
fn guard_on_data_enum() {
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
  var S: Shape := Shape.Circle(15.0);
  case S of
    Shape.Circle(R) if R > 10.0:
      Std.Console.WriteLn('large circle');
    Shape.Circle(R):
      Std.Console.WriteLn('small circle');
    Shape.Rectangle(W, H) if W = H:
      Std.Console.WriteLn('square');
    Shape.Rectangle(W, H):
      Std.Console.WriteLn('rectangle');
    Shape.Point:
      Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["large circle"]);
}

#[test]
fn guard_on_data_enum_fallthrough() {
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
  var S: Shape := Shape.Circle(3.0);
  case S of
    Shape.Circle(R) if R > 10.0:
      Std.Console.WriteLn('large');
    Shape.Circle(R):
      Std.Console.WriteLn('small');
    Shape.Rectangle(W, H):
      Std.Console.WriteLn('rect');
    Shape.Point:
      Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["small"]);
}

#[test]
fn guard_on_data_enum_square() {
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
  var S: Shape := Shape.Rectangle(5.0, 5.0);
  case S of
    Shape.Circle(R) if R > 10.0:
      Std.Console.WriteLn('large circle');
    Shape.Circle(R):
      Std.Console.WriteLn('small circle');
    Shape.Rectangle(W, H) if W = H:
      Std.Console.WriteLn('square');
    Shape.Rectangle(W, H):
      Std.Console.WriteLn('rectangle');
    Shape.Point:
      Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["square"]);
}

#[test]
fn guard_on_fieldless_variant_with_else() {
    let out = compile_and_run(
        "\
program T;
type
  Shape = enum
    Circle(Radius: real);
    Point;
  end;
begin
  var S: Shape := Shape.Point;
  case S of
    Shape.Circle(R):
      Std.Console.WriteLn('circle');
    Shape.Point:
      Std.Console.WriteLn('point')
  end
end.",
    );
    assert_eq!(out.lines, vec!["point"]);
}

#[test]
fn guard_on_data_enum_multiple_guarded_same_variant() {
    let out = compile_and_run(
        "\
program T;
type Val = enum Num(N: integer); end;
begin
  var V: Val := Val.Num(50);
  case V of
    Val.Num(N) if N > 100:
      Std.Console.WriteLn('big');
    Val.Num(N) if N > 75:
      Std.Console.WriteLn('medium');
    Val.Num(N):
      Std.Console.WriteLn('small ' + Std.Conv.IntToStr(N))
  end
end.",
    );
    assert_eq!(out.lines, vec!["small 50"]);
}
