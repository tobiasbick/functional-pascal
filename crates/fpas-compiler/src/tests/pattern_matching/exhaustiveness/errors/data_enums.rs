use super::*;

#[test]
fn exhaustiveness_error_data_enum_missing_variant() {
    let err = compile_err(
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
    Shape.Rectangle(W, H): Std.Console.WriteLn('rect')
  end
end.",
    );
    assert_eq!(err.code, fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE);
    assert!(err.message.contains("Point"));
}
