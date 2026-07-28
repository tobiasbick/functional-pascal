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
fn unit_record_fields_preserve_per_member_visibility() {
    let unit = parse_unit_ok(
        "unit Demo.Types; \
         type Counter = record \
           private Value: integer; \
           public Step: integer; \
           LabelText: string; \
         end;",
    );
    let Decl::TypeDef(type_def) = &unit.declarations[0] else {
        panic!("expected TypeDef");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected Record");
    };

    assert_eq!(record.fields[0].visibility, Visibility::Private);
    assert_eq!(record.fields[1].visibility, Visibility::Public);
    assert_eq!(record.fields[2].visibility, Visibility::Public);
}

#[test]
fn record_field_visibility_is_rejected_in_program_files() {
    let (program, errors) = parse_with_errors(
        "program T; \
         type Counter = record private Value: integer; end; \
         begin end.",
    );
    let Decl::TypeDef(type_def) = &program.declarations[0] else {
        panic!("expected TypeDef");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected Record");
    };

    assert_eq!(record.fields[0].visibility, Visibility::Private);
    assert!(errors.iter().any(|diagnostic| {
        matches!(
            diagnostic,
            ParseDiagnostic::Parser(error) if error.code == fpas_diagnostics::codes::PARSE_INVALID_VISIBILITY
        )
    }));
}

#[test]
fn invalid_record_field_recovery_preserves_following_field() {
    let (p, errors) =
        parse_with_errors("program T; type Point = record X: real; 123; Y: real; end; begin end.");
    assert!(!errors.is_empty());
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.fields.len(), 3);
                assert_eq!(r.fields[0].name, "X");
                assert_eq!(r.fields[2].name, "Y");
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn invalid_record_field_recovery_preserves_following_function_declaration() {
    let (p, errors) = parse_with_errors(
        "program T; \
         type Point = record X: real; 123; end; \
         function Answer(): integer; begin return 42 end; \
         begin end.",
    );
    assert!(!errors.is_empty());
    assert_eq!(p.declarations.len(), 2);
    assert!(matches!(&p.declarations[0], Decl::TypeDef(_)));
    assert!(matches!(&p.declarations[1], Decl::Function(_)));
}
