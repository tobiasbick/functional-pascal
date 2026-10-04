use super::*;

#[test]
fn generic_data_headers_and_nested_type_arguments() {
    let program = parse_ok(
        "program Main;
        type Pair of (K: Equatable, V) = record Key: K; Value: V; end record;
        type Lookup of (T) = enum Found(Value: T); Missing; end enum;
        type Entries = array of (Pair of (string, Lookup of (integer)));
        begin null; end program;",
    );
    let Decl::TypeDef(pair) = &program.declarations[0] else {
        panic!("pair declaration");
    };
    assert_eq!(pair.type_params.len(), 2);
    assert_eq!(pair.type_params[0].constraint.as_deref(), Some("Equatable"));
    let Decl::TypeDef(entries) = &program.declarations[2] else {
        panic!("alias declaration");
    };
    let TypeBody::Alias(TypeExpr::Array(element, _)) = &entries.body else {
        panic!("array alias");
    };
    let TypeExpr::Named {
        id,
        arguments,
        span,
    } = element.as_ref()
    else {
        panic!("pair application");
    };
    assert_eq!(id.parts, ["Pair"]);
    assert_eq!(arguments.len(), 2);
    assert!(span.length > id.span.length);
    assert!(matches!(&arguments[1], TypeExpr::Named { arguments, .. } if arguments.len() == 1));
}

#[test]
fn canonical_builtin_type_lists_and_routine_parameters() {
    parse_ok(
        "program Main;
        type Items = array of (dict of (string, Option of (Result of (integer, string))));
        function Identity of (T)(Value: T): T;
        begin return Value; end function;
        procedure Accept(Value: channel of (task of (integer)));
        begin null; end procedure;
        begin null; end program;",
    );
}

#[test]
fn generic_lists_reject_empty_and_trailing_arguments() {
    for fragment in [
        "type Box of () = record end record;",
        "type Box of (T,) = record Value: T; end record;",
        "type Item = Box of ();",
        "type Item = Box of (integer,);",
    ] {
        let (_, errors) = parse_with_errors(&format!(
            "program Main; {fragment} begin null; end program;"
        ));
        assert!(!errors.is_empty(), "{fragment}");
    }
}
