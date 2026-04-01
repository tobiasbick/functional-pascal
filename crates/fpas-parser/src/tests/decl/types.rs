use super::*;

#[test]
fn record_type() {
    let p = parse_ok("program T; type Point = record X: real; Y: real; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Point");
            match &td.body {
                TypeBody::Record(r) => {
                    assert_eq!(r.fields.len(), 2);
                    assert_eq!(r.fields[0].name, "X");
                    assert_eq!(r.fields[1].name, "Y");
                }
                _ => panic!("expected Record"),
            }
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_type() {
    let p = parse_ok("program T; type Color = enum Red; Green; Blue; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 3);
                assert_eq!(e.members[0].name, "Red");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_with_values() {
    let p = parse_ok("program T; type Suit = enum Hearts = 1; Diamonds = 2; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members[0].value, Some(1));
                assert_eq!(e.members[1].value, Some(2));
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_member_non_integer_value_uses_parser_code() {
    let (_, errors) =
        parse_with_errors("program T; type Suit = enum Hearts = true; end; begin end.");
    let error = errors
        .iter()
        .find_map(|diagnostic| match diagnostic {
            ParseDiagnostic::Parser(error) => Some(error),
            ParseDiagnostic::Lexer(_) => None,
        })
        .expect("expected parser diagnostic");
    assert_eq!(error.code, PARSE_EXPECTED_TOKEN);
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("integer literals")),
        "expected explicit enum-value help text"
    );
}

#[test]
fn type_alias() {
    let p = parse_ok("program T; type Name = string; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Name");
            assert!(matches!(&td.body, TypeBody::Alias(_)));
        }
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn array_type() {
    let p = parse_ok("program T; var Xs: array of integer := []; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Array(inner, _) => {
                assert!(matches!(inner.as_ref(), TypeExpr::Named { .. }));
            }
            _ => panic!("expected array type"),
        },
        _ => panic!("expected Var"),
    }
}

// ── Enums with Associated Data ──────────────────────────────

#[test]
fn enum_data_single_field() {
    let p = parse_ok("program T; type Wrapper = enum Val(X: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 1);
                assert_eq!(e.members[0].name, "Val");
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[0].fields[0].name, "X");
                assert!(e.members[0].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_multiple_fields() {
    let p = parse_ok(
        "program T; type Shape = enum Circle(Radius: real); Rect(W: real; H: real); end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert_eq!(e.members[0].fields.len(), 1);
                assert_eq!(e.members[1].fields.len(), 2);
                assert_eq!(e.members[1].fields[0].name, "W");
                assert_eq!(e.members[1].fields[1].name, "H");
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_mixed_simple_and_data_variants() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert_eq!(e.members.len(), 2);
                assert!(e.members[0].fields.is_empty());
                assert_eq!(e.members[1].fields.len(), 1);
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_data_fieldless_has_no_backing_value() {
    let p = parse_ok("program T; type Token = enum Eof; Number(V: integer); end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Enum(e) => {
                assert!(e.members[0].value.is_none());
                assert!(e.members[1].value.is_none());
            }
            _ => panic!("expected Enum"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn enum_variant_cannot_mix_fields_with_backing_value() {
    let (_, errors) =
        parse_with_errors("program T; type Shape = enum Circle(Radius: real) = 1; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parser error when mixing enum fields with a backing value"
    );
}

// ── Generics (parser-level) ────────────────────────────────

// Generic type params on type defs produce a parse error.

#[test]
fn generic_record_type_params_not_allowed() {
    let (_, errors) =
        parse_with_errors("program T; type Box<T> = record Value: integer; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic type definition"
    );
}

#[test]
fn generic_enum_type_params_not_allowed() {
    let (_, errors) =
        parse_with_errors("program T; type Maybe<T> = enum Just; Nothing; end; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic enum definition"
    );
}

#[test]
fn generic_type_alias_of_syntax_not_allowed() {
    let (_, errors) =
        parse_with_errors("program T; type Foo = integer; var X: Foo of integer := 0; begin end.");
    assert!(
        !errors.is_empty(),
        "expected parse error for generic type argument syntax"
    );
}

// ── Record Methods (parser-level) ──────────────────────────

// 7. Record with function method
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

// 8. Record with procedure method
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

// 9. Record with multiple methods (both function and procedure)
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

// 10. Record method with its own generic type parameters
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
