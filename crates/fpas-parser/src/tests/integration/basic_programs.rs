use super::*;

#[test]
fn hello_world() {
    let p = parse_ok(
        "\
program Hello;
uses
  Std.Console;
begin
  Std.Console.WriteLn('Hello, World!')
end.",
    );
    assert_eq!(p.name, "Hello");
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["Std", "Console"]);
    assert_eq!(p.body.len(), 1);
    assert!(matches!(&p.body[0], Stmt::Call { .. }));
}

#[test]
fn full_program() {
    let p = parse_ok(
        "\
program Calculator;
uses Std.Console;

type Op = enum
  OpAdd;
  OpSub;
  OpMul;
  OpDiv;
end;

function Calculate(A: integer; B: integer; Operation: Op): integer;
begin
  case Operation of
    OpAdd: return A + B;
    OpSub: return A - B;
    OpMul: return A * B;
    OpDiv: return A div B
  end
end;

begin
  var Answer: integer := Calculate(10, 3, OpAdd);
  Std.Console.WriteLn(Answer)
end.",
    );
    assert_eq!(p.name, "Calculator");
    assert_eq!(p.declarations.len(), 2);
    assert_eq!(p.body.len(), 2);
}