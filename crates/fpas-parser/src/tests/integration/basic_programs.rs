use super::*;

#[test]
fn hello_world() {
    let p = parse_ok(
        "\
program Hello;
uses
  Std.Console;
begin
  Std.Console.WriteLn('Hello, World!');
end.",
    );
    assert_eq!(p.name, "Hello");
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].unit.parts, vec!["Std", "Console"]);
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
end enum;

function Calculate(A: integer; B: integer; Operation: Op): integer;
begin
  case Operation of
    when OpAdd: return A + B;
    when OpSub: return A - B;
    when OpMul: return A * B;
    when OpDiv: return A div B;
  end case;
end function;

begin
  var Answer: integer := Calculate(10, 3, OpAdd);
  Std.Console.WriteLn(Answer);
end.",
    );
    assert_eq!(p.name, "Calculator");
    assert_eq!(p.declarations.len(), 2);
    assert_eq!(p.body.len(), 2);
}
