use super::*;

#[test]
fn function_forward() {
    let p =
        parse_ok("program T; function Add(A: integer; B: integer): integer; forward; begin end.");
    match &p.declarations[0] {
        Decl::Function(f) => {
            assert_eq!(f.name, "Add");
            assert_eq!(f.params.len(), 2);
            assert!(matches!(f.body, FuncBody::Forward));
        }
        _ => panic!("expected Function"),
    }
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
                _ => panic!("expected Block body"),
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
            _ => panic!("expected Block"),
        },
        _ => panic!("expected Function"),
    }
}
