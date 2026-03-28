use super::*;

#[test]
fn guard_on_integer_classify() {
    let out = compile_and_run(
        "\
program T;

function Classify(N: integer): string;
begin
  case N of
    0:
      return 'zero';
    N if N > 0:
      return 'positive';
    N if N < 0:
      return 'negative'
  end;
  return 'unknown'
end;

begin
  Std.Console.WriteLn(Classify(-5));
  Std.Console.WriteLn(Classify(0));
  Std.Console.WriteLn(Classify(42))
end.",
    );
    assert_eq!(out.lines, vec!["negative", "zero", "positive"]);
}

#[test]
fn guard_on_integer_ranges() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 15;
  case X of
    0..100 if X > 10:
      Std.Console.WriteLn('big');
    0..100:
      Std.Console.WriteLn('small')
  else
    Std.Console.WriteLn('out of range')
  end
end.",
    );
    assert_eq!(out.lines, vec!["big"]);
}

#[test]
fn guard_with_complex_boolean_expression() {
    let out = compile_and_run(
        "\
program T;
begin
  var X: integer := 15;
  case X of
    X if (X > 10) and (X < 20):
      Std.Console.WriteLn('teen');
    X if (X >= 20) or (X <= 10):
      Std.Console.WriteLn('other')
  else
    Std.Console.WriteLn('else')
  end
end.",
    );
    assert_eq!(out.lines, vec!["teen"]);
}
