use super::*;

#[test]
fn record_with_function_method() {
    let p = parse_ok(
        "program T; type Num = record V: integer; \
         function Double(Self: Num): integer; begin return Self.V * 2 end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.fields.len(), 1);
                assert_eq!(r.methods.len(), 1);
                assert!(matches!(&r.methods[0], RecordMethod::Function(_)));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_with_procedure_method() {
    let p = parse_ok(
        "program T; type Greeter = record Name: string; \
         procedure SayHello(Self: Greeter); begin Std.Console.WriteLn('hi') end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.methods.len(), 1);
                assert!(matches!(&r.methods[0], RecordMethod::Procedure(_)));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_with_multiple_methods() {
    let p = parse_ok(
        "program T; type Rect = record W: integer; H: integer; \
         function Area(Self: Rect): integer; begin return Self.W * Self.H end; \
         procedure Print(Self: Rect); begin Std.Console.WriteLn(Self.W) end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.fields.len(), 2);
                assert_eq!(r.methods.len(), 2);
                assert!(matches!(&r.methods[0], RecordMethod::Function(_)));
                assert!(matches!(&r.methods[1], RecordMethod::Procedure(_)));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_with_generic_function_method() {
    let p = parse_ok(
        "program T; type Box = record Value: integer; \
         function Map<R>(Self: Box; F: function(X: integer): R): R; \
         begin return F(Self.Value) end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => match &r.methods[0] {
                RecordMethod::Function(f) => {
                    assert_eq!(f.name, "Map");
                    assert_eq!(f.type_params.len(), 1);
                    assert_eq!(f.type_params[0].name, "R");
                }
                _ => panic!("expected function method"),
            },
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}