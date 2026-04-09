use super::*;

#[test]
fn minimal_program() {
    let p = parse_ok("program Hello; begin end.");
    assert_eq!(p.name, "Hello");
    assert!(p.uses.is_empty());
    assert!(p.declarations.is_empty());
    assert!(p.body.is_empty());
}

#[test]
fn program_with_uses() {
    let p = parse_ok("program Test; uses Std.Console, Std.Math; begin end.");
    assert_eq!(p.uses.len(), 2);
    assert_eq!(p.uses[0].parts, vec!["Std", "Console"]);
    assert_eq!(p.uses[1].parts, vec!["Std", "Math"]);
}

#[test]
fn program_with_uses_std_array() {
    let p = parse_ok("program T; uses Std.Array; begin end.");
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["Std", "Array"]);
}

#[test]
fn program_with_uses_std_array_lowercase_unit_keyword() {
    let p = parse_ok("program T; uses Std.array; begin end.");
    assert_eq!(p.uses.len(), 1);
    assert_eq!(p.uses[0].parts, vec!["Std", "Array"]);
}

#[test]
fn program_with_const() {
    let p = parse_ok("program T; const Pi: real := 3.14; begin end.");
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
    let p = parse_ok("program T; const A: integer := 1; B: integer := 2; begin end.");
    assert_eq!(p.declarations.len(), 2);
}

#[test]
fn program_with_var() {
    let p = parse_ok("program T; var X: integer := 42; begin end.");
    assert_eq!(p.declarations.len(), 1);
    match &p.declarations[0] {
        Decl::Var(v) => {
            assert_eq!(v.name, "X");
            assert!(matches!(v.value, Expr::Integer(42, _)));
        }
        _ => panic!("expected Var"),
    }
}

#[test]
fn program_with_mutable_var() {
    let p = parse_ok("program T; mutable var Count: integer := 0; begin end.");
    assert_eq!(p.declarations.len(), 1);
    assert!(matches!(&p.declarations[0], Decl::MutableVar(_)));
}
