use super::compile_and_run;
#[test]
fn case_result_destructure() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Ok(42);
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["42"]);
}
#[test]
fn case_result_err_branch() {
    let out = compile_and_run(
        "program T;
var R: Result of integer, string := Error('failed');
begin
  case R of
    Ok(V):  Std.Console.WriteLn(V);
    Error(E): Std.Console.WriteLn(E)
  end
end.",
    );
    assert_eq!(out.lines, vec!["failed"]);
}
#[test]
fn case_option_destructure() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := Some(99);
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["99"]);
}
#[test]
fn case_option_none_branch() {
    let out = compile_and_run(
        "program T;
var O: Option of integer := None;
begin
  case O of
    Some(V): Std.Console.WriteLn(V);
    None:    Std.Console.WriteLn('nothing')
  end
end.",
    );
    assert_eq!(out.lines, vec!["nothing"]);
}
