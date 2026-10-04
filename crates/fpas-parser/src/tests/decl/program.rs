use super::*;

#[test]
fn minimal_program() {
    let p = parse_ok(r#"program Hello; begin null; end program;"#);
    assert_eq!(p.name, "Hello");
    assert!(p.uses.is_empty());
    assert!(p.declarations.is_empty());
    assert!(matches!(p.body.as_slice(), [Stmt::Null(_)]));
}

#[test]
fn program_with_uses() {
    let p = parse_ok(
        r#"program Test;  uses Std.Console as Console; uses Std.Math as Math; begin null; end program;"#,
    );
    assert_eq!(p.uses.len(), 2);
    assert_eq!(p.uses[0].parts, vec!["Std", "Console"]);
    assert_eq!(p.uses[1].parts, vec!["Std", "Math"]);
}

#[test]
fn program_with_uses_std_array() {
    let p = parse_ok(r#"program T;  uses Std.Arrays as Arrays; begin null; end program;"#);
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["Std", "Arrays"]);
}

#[test]
fn program_with_uses_std_arrays_case_insensitively() {
    let p = parse_ok(r#"program T;  uses std.arrays as arrays; begin null; end program;"#);
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["std", "arrays"]);
}

#[test]
fn program_with_const() {
    let p = parse_ok(r#"program T;  const Pi: real := 3.14; begin null; end program;"#);
    assert_eq!(p.declarations.len(), 1);
    match &p.declarations[0] {
        Decl::Const(c) => {
            assert_eq!(c.name, "Pi");
            assert!(matches!(c.value, Expr::Real(v, _) if (v - 3.14).abs() < 1e-10));
        }
        _ => panic!("expected Const"),
    }
}

#[test]
fn program_with_multiple_consts() {
    let p = parse_ok(
        r#"program T;  const A: integer := 1; const B: integer := 2; begin null; end program;"#,
    );
    assert_eq!(p.declarations.len(), 2);
}

#[test]
fn program_with_var() {
    let p = parse_ok(r#"program T;  const X: integer := 42; begin null; end program;"#);
    assert_eq!(p.declarations.len(), 1);
    match &p.declarations[0] {
        Decl::Const(v) => {
            assert_eq!(v.name, "X");
            assert!(matches!(v.value, Expr::Integer(42, _)));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn program_with_mutable_var() {
    let p = parse_ok(r#"program T;    var Count: integer := 0; begin null; end program;"#);
    assert_eq!(p.declarations.len(), 1);
    assert!(matches!(&p.declarations[0], Decl::Var(_)));
}
