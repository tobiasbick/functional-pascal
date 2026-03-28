use super::*;

// ===========================================================================
// Guard clauses — Result/Option destructuring
// ===========================================================================

#[test]
fn guard_on_result_destructure() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Ok(42);
  case R of
    Ok(V) if V > 100:
      Std.Console.WriteLn('big');
    Ok(V):
      Std.Console.WriteLn('small ' + Std.Conv.IntToStr(V));
    Error(E):
      Std.Console.WriteLn('err')
  end
end.",
    );
    assert_eq!(out.lines, vec!["small 42"]);
}

#[test]
fn guard_on_result_err_arm() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Error('timeout');
  case R of
    Ok(V):
      Std.Console.WriteLn('ok');
    Error(E) if E = 'timeout':
      Std.Console.WriteLn('timed out');
    Error(E):
      Std.Console.WriteLn('other error: ' + E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["timed out"]);
}

#[test]
fn guard_on_result_err_fallthrough() {
    let out = compile_and_run(
        "\
program T;
begin
  var R: Result of integer, string := Error('unknown');
  case R of
    Ok(V):
      Std.Console.WriteLn('ok');
    Error(E) if E = 'timeout':
      Std.Console.WriteLn('timed out');
    Error(E):
      Std.Console.WriteLn('other: ' + E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["other: unknown"]);
}

#[test]
fn guard_on_option_destructure() {
    let out = compile_and_run(
        "\
program T;
begin
  var O: Option of integer := Some(200);
  case O of
    Some(V) if V > 100:
      Std.Console.WriteLn('big ' + Std.Conv.IntToStr(V));
    Some(V):
      Std.Console.WriteLn('small');
    None:
      Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["big 200"]);
}

#[test]
fn guard_on_option_fallthrough_to_none() {
    let out = compile_and_run(
        "\
program T;
begin
  var O: Option of integer := None;
  case O of
    Some(V) if V > 0:
      Std.Console.WriteLn('positive');
    Some(V):
      Std.Console.WriteLn('non-positive');
    None:
      Std.Console.WriteLn('none')
  end
end.",
    );
    assert_eq!(out.lines, vec!["none"]);
}
