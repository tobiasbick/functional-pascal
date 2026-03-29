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

// 1. Generic record with single type param
#[test]
fn generic_record_single_param() {
    let p = parse_ok("program T; type Box<T> = record Value: T; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Box");
            assert_eq!(td.type_params.len(), 1);
            assert_eq!(td.type_params[0].name, "T");
            assert!(td.type_params[0].constraint.is_none());
            match &td.body {
                TypeBody::Record(r) => assert_eq!(r.fields.len(), 1),
                _ => panic!("expected Record"),
            }
        }
        _ => panic!("expected TypeDef"),
    }
}

// 2. Generic record with multiple type params
#[test]
fn generic_record_multiple_params() {
    let p = parse_ok("program T; type Pair<A, B> = record First: A; Second: B; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.type_params.len(), 2);
            assert_eq!(td.type_params[0].name, "A");
            assert_eq!(td.type_params[1].name, "B");
        }
        _ => panic!("expected TypeDef"),
    }
}

// 3. Generic record with constraint
#[test]
fn generic_record_with_constraint() {
    let p = parse_ok("program T; type Ordered<T: Comparable> = record Value: T; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.type_params.len(), 1);
            assert_eq!(td.type_params[0].name, "T");
            assert_eq!(td.type_params[0].constraint.as_deref(), Some("Comparable"));
        }
        _ => panic!("expected TypeDef"),
    }
}

// 4. Generic record with mixed constrained/unconstrained params
#[test]
fn generic_record_mixed_constraints() {
    let p = parse_ok(
        "program T; type Entry<K: Comparable, V> = record Key: K; Value: V; end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.type_params.len(), 2);
            assert_eq!(td.type_params[0].name, "K");
            assert_eq!(td.type_params[0].constraint.as_deref(), Some("Comparable"));
            assert_eq!(td.type_params[1].name, "V");
            assert!(td.type_params[1].constraint.is_none());
        }
        _ => panic!("expected TypeDef"),
    }
}

// 5. Generic enum
#[test]
fn generic_enum_single_param() {
    let p = parse_ok("program T; type Maybe<T> = enum Just(Value: T); Nothing; end; begin end.");
    match &p.declarations[0] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "Maybe");
            assert_eq!(td.type_params.len(), 1);
            assert_eq!(td.type_params[0].name, "T");
            match &td.body {
                TypeBody::Enum(e) => {
                    assert_eq!(e.members.len(), 2);
                    assert_eq!(e.members[0].fields.len(), 1);
                    assert!(e.members[1].fields.is_empty());
                }
                _ => panic!("expected Enum"),
            }
        }
        _ => panic!("expected TypeDef"),
    }
}

// 6. Generic type alias (alias with type args via `of`)
#[test]
fn generic_type_alias_of_syntax() {
    let p = parse_ok(
        "program T; type Box<T> = record Value: T; end; type IntBox = Box of integer; begin end.",
    );
    match &p.declarations[1] {
        Decl::TypeDef(td) => {
            assert_eq!(td.name, "IntBox");
            assert!(td.type_params.is_empty());
            match &td.body {
                TypeBody::Alias(TypeExpr::Named { type_args, .. }) => {
                    assert_eq!(type_args.len(), 1);
                }
                _ => panic!("expected Alias with Named type"),
            }
        }
        _ => panic!("expected TypeDef"),
    }
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
