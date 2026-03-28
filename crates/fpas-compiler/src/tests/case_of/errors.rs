use super::super::*;

#[test]
fn case_type_mismatch_string_label_on_integer() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    'hello': Std.Console.WriteLn('bad')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("type") || msg.contains("mismatch") || msg.contains("compat"),
        "expected type error, got: {}",
        err.message
    );
}

#[test]
fn case_type_mismatch_integer_label_on_string() {
    let err = compile_err(
        "\
program T;
begin
  var S: string := 'hello';
  case S of
    42: Std.Console.WriteLn('bad')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("type") || msg.contains("mismatch") || msg.contains("compat"),
        "expected type error, got: {}",
        err.message
    );
}

#[test]
fn case_guard_non_boolean_rejected() {
    let err = compile_err(
        "\
program T;
begin
  var X: integer := 5;
  case X of
    X if 42: Std.Console.WriteLn('bad')
  end
end.",
    );
    let msg = err.message.to_lowercase();
    assert!(
        msg.contains("boolean") || msg.contains("guard"),
        "expected guard type error, got: {}",
        err.message
    );
}
