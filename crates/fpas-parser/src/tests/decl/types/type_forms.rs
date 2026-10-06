use super::*;

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
fn event_keyword_cannot_be_a_type_name() {
    let (_, errors) = parse_with_errors("program T; type Event = string; begin end.");
    assert!(!errors.is_empty());
}

#[test]
fn property_keyword_cannot_be_a_variable_name() {
    let (_, errors) = parse_with_errors("program T; var Property: integer := 1; begin end.");
    assert!(!errors.is_empty());
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

#[test]
fn channel_type() {
    let p = parse_ok("program T; var Messages: channel of string := Value; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Channel(inner, _) => {
                assert!(matches!(inner.as_ref(), TypeExpr::Named { .. }));
            }
            _ => panic!("expected channel type"),
        },
        _ => panic!("expected Var"),
    }
}

#[test]
fn typed_task_type() {
    let p = parse_ok("program T; var Job: Task of result of integer, string := Value; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Task(inner, _) => {
                assert!(matches!(inner.as_ref(), TypeExpr::Result { .. }));
            }
            _ => panic!("expected task type"),
        },
        _ => panic!("expected Var"),
    }
}

#[test]
fn bare_task_remains_a_named_type() {
    let p = parse_ok("program T; var Job: task := Value; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => assert!(matches!(&v.type_expr, TypeExpr::Named { .. })),
        _ => panic!("expected Var"),
    }
}

#[test]
fn other_named_types_still_reject_generic_arguments() {
    let (_, errors) =
        parse_with_errors("program T; var Items: Queue of integer := Value; begin end.");
    assert!(!errors.is_empty());
}

#[test]
fn task_keyword_cannot_be_a_name() {
    for source in [
        "program T; var Task: integer := 1; begin end.",
        "program T; function F(Task: integer): integer; begin return Task end; begin end.",
        "program T; type Job = record Task: integer; end record; begin end.",
        "program T; uses Std.Task; begin end.",
    ] {
        let (_, errors) = parse_with_errors(source);
        assert!(!errors.is_empty(), "{source}");
    }
}

#[test]
fn task_keyword_types_nest_in_other_type_forms() {
    let p = parse_ok("program T; var Jobs: array of TASK of option of task := []; begin end.");
    match &p.declarations[0] {
        Decl::Var(v) => match &v.type_expr {
            TypeExpr::Array(inner, _) => match inner.as_ref() {
                TypeExpr::Task(result, _) => match result.as_ref() {
                    TypeExpr::Option { inner_type, .. } => {
                        assert!(matches!(inner_type.as_ref(), TypeExpr::Named { .. }));
                    }
                    _ => panic!("expected option result type"),
                },
                _ => panic!("expected task element type"),
            },
            _ => panic!("expected array type"),
        },
        _ => panic!("expected Var"),
    }
}
