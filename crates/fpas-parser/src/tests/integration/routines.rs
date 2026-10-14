use super::*;

#[test]
fn fibonacci() {
    let p = parse_ok(
        "program Fib;\nuses Std.Console as Console;\n\nfunction Fibonacci(N: integer): integer;\nbegin\n  if N <= 1 then\n    return N;\n  else\n    return Fibonacci(N - 1) + Fibonacci(N - 2); end if;\nend function;\n\nbegin\n  Console.WriteLn(Fibonacci(10));\nend.",
    );
    assert_eq!(p.name, "Fib");
    assert_eq!(p.declarations.len(), 1);
    assert!(matches!(&p.declarations[0], Decl::Function(_)));
}

#[test]
fn nested_mutual_recursion_even_odd() {
    let p = parse_ok(
        "\
program T;

function IsEven(N: integer): boolean;
  function IsOdd(X: integer): boolean;
  begin
    if X = 0 then return false;
    else return IsEven(X - 1); end if;
  end function;
begin
  if N = 0 then return true;
  else return IsOdd(N - 1); end if;
end function;

begin
  return;
end.",
    );
    assert_eq!(p.declarations.len(), 1);
    match &p.declarations[0] {
        Decl::Function(f) => match &f.body {
            FuncBody::Block { nested, .. } => assert_eq!(nested.len(), 1),
        },
        _ => panic!("expected Function"),
    }
}
