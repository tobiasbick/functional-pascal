use super::*;

#[test]
fn nested_pattern_fieldless_inner_variant() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Token = enum
    Eof;
    Plus;
    Number(V: integer);
  end;
  Tree = enum
    Leaf(T: Token);
    Node(Op: Token; Left: Tree; Right: Tree);
  end;

begin
  var T: Tree := Tree.Leaf(Token.Eof);
  case T of
    Tree.Leaf(Token.Eof):
      WriteLn('eof');
    Tree.Leaf(Token.Plus):
      WriteLn('plus');
    Tree.Leaf(Token.Number(N)):
      WriteLn('number');
  else
    WriteLn('node')
  end
end.",
    );
    assert_eq!(out.lines, vec!["eof"]);
}

#[test]
fn nested_pattern_fieldless_inner_second_variant() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console;

type
  Token = enum
    Eof;
    Plus;
    Number(V: integer);
  end;
  Tree = enum
    Leaf(T: Token);
    Node(Op: Token; Left: Tree; Right: Tree);
  end;

begin
  var T: Tree := Tree.Leaf(Token.Plus);
  case T of
    Tree.Leaf(Token.Eof):
      WriteLn('eof');
    Tree.Leaf(Token.Plus):
      WriteLn('plus');
    Tree.Leaf(Token.Number(N)):
      WriteLn('number');
  else
    WriteLn('node')
  end
end.",
    );
    assert_eq!(out.lines, vec!["plus"]);
}
