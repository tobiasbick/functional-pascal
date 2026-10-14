use super::*;

#[test]
fn hello_world() {
    let p = parse_ok(
        "program Hello;\nuses\n  Std.Console as Console;\nbegin\n  Console.WriteLn('Hello, World!');\nend.",
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
        "program Calculator;\nuses Std.Console as Console;\n\ntype Op = enum\n  OpAdd;\n  OpSub;\n  OpMul;\n  OpDiv;\nend enum;\n\nfunction Calculate(A: integer; B: integer; Operation: Op): integer;\nbegin\n  case Operation of\n    when Op.OpAdd: return A + B;\n    when Op.OpSub: return A - B;\n    when Op.OpMul: return A * B;\n    when Op.OpDiv: return A div B;\n  end case;\nend function;\n\nbegin\n  var Answer: integer := Calculate(10, 3, Op.OpAdd);\n  Console.WriteLn(Answer);\nend.",
    );
    assert_eq!(p.name, "Calculator");
    assert_eq!(p.declarations.len(), 2);
    assert_eq!(p.body.len(), 2);
}
