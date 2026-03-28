use super::*;

#[test]
fn enum_data_string_fields() {
    let out = compile_and_run(
        "\
program EnumStr;
uses Std.Console;
type Msg = enum
  Text(Body: string);
  Tagged(Tag: string; Body: string);
end;
begin
  var M: Msg := Msg.Tagged('info', 'hello world');
  case M of
    Msg.Text(B): WriteLn(B);
    Msg.Tagged(T, B): begin
      WriteLn(T);
      WriteLn(B)
    end
  end
end.",
    );
    assert_eq!(out.lines, vec!["info", "hello world"]);
}

#[test]
fn enum_data_boolean_field() {
    let out = compile_and_run(
        "\
program EnumBool;
uses Std.Console;
type Flag = enum
  On(Verbose: boolean);
  Off;
end;
begin
  var F: Flag := Flag.On(true);
  case F of
    Flag.On(V): WriteLn(V);
    Flag.Off: WriteLn('off')
  end
end.",
    );
    assert_eq!(out.lines, vec!["true"]);
}

#[test]
fn enum_data_compute_with_bindings() {
    let out = compile_and_run(
        "\
program EnumCompute;
uses Std.Console, Std.Conv;
type Shape = enum
  Rect(W: real; H: real);
end;
begin
  var R: Shape := Shape.Rect(3.0, 4.0);
  case R of
    Shape.Rect(W, H): WriteLn(Std.Conv.RealToStr(W * H))
  end
end.",
    );
    assert_eq!(out.lines, vec!["12"]);
}

#[test]
fn enum_data_reuse_binding_names_across_arms() {
    let out = compile_and_run(
        "\
program EnumReuse;
uses Std.Console;
type Val = enum
  IntVal(X: integer);
  RealVal(X: real);
end;
begin
  var V: Val := Val.RealVal(9.5);
  case V of
    Val.IntVal(X): WriteLn(X);
    Val.RealVal(X): WriteLn(X)
  end
end.",
    );
    assert_eq!(out.lines, vec!["9.5"]);
}
