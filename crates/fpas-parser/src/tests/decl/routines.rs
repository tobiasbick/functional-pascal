use super::{parse_ok, parse_with_errors};
use crate::ast::*;

#[test]
fn function_rejects_legacy_forward_token() {
    let (_p, errors) = parse_with_errors(
        "program T; function Add(A: integer; B: integer): integer; forward; begin end.",
    );
    assert!(
        !errors.is_empty(),
        "expected parse errors for `forward` after header, got none"
    );
}

#[test]
fn function_with_body() {
    let p = parse_ok(
        "program T; \
         function Add(A: integer; B: integer): integer; \
         begin \
           return A + B \
         end; \
         begin end.",
    );
    match &p.declarations[0] {
        Decl::Function(f) => {
            assert_eq!(f.name, "Add");
            match &f.body {
                FuncBody::Block { nested, stmts } => {
                    assert!(nested.is_empty());
                    assert_eq!(stmts.len(), 1);
                }
            }
        }
        _ => panic!("expected Function"),
    }
}

#[test]
fn procedure_declaration() {
    let p = parse_ok(
        "program T; \
         procedure Greet(Name: string); \
         begin \
           Std.Console.WriteLn(Name) \
         end; \
         begin end.",
    );
    match &p.declarations[0] {
        Decl::Procedure(proc) => {
            assert_eq!(proc.name, "Greet");
            assert_eq!(proc.params.len(), 1);
        }
        _ => panic!("expected Procedure"),
    }
}

#[test]
fn mutable_param() {
    let p = parse_ok(
        "program T; \
         procedure Inc(mutable X: integer); \
         begin \
           X := X + 1 \
         end; \
         begin end.",
    );
    match &p.declarations[0] {
        Decl::Procedure(proc) => {
            assert!(proc.params[0].mutable);
        }
        _ => panic!("expected Procedure"),
    }
}

#[test]
fn nested_function() {
    let p = parse_ok(
        "program T; \
         function Outer(): integer; \
           function Inner(): integer; \
           begin return 1 end; \
         begin return Inner() end; \
         begin end.",
    );
    match &p.declarations[0] {
        Decl::Function(f) => match &f.body {
            FuncBody::Block { nested, .. } => {
                assert_eq!(nested.len(), 1);
            }
        },
        _ => panic!("expected Function"),
    }
}
