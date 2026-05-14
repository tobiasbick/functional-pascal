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
