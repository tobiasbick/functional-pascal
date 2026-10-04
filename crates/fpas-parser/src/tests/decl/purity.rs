//! Explicit purity syntax applies only to functions and function types.

use super::*;

#[test]
fn pure_declarations_types_and_closures_preserve_capability() {
    let program = parse_ok("program Main;
        type Callback = pure function(Value: integer): integer;
        PuRe FuNcTiOn Identity(Value: integer): integer;
        begin return Value; end function;
        begin const F: Callback := pure function(Value: integer): integer begin return Value; end function; end program;");
    let Decl::TypeDef(definition) = &program.declarations[0] else {
        panic!("type")
    };
    assert!(matches!(
        definition.body,
        TypeBody::Alias(TypeExpr::FunctionType { pure: true, .. })
    ));
    let Decl::Function(function) = &program.declarations[1] else {
        panic!("function")
    };
    assert!(function.pure);
    let Stmt::Const(binding) = &program.body[0] else {
        panic!("binding")
    };
    let Expr::Closure(closure) = &binding.value else {
        panic!("closure")
    };
    assert!(closure.pure && closure.is_function);
    let unit = parse_unit_ok(
        "unit Demo.Purity; public pure function Value(): integer; begin return 1; end function; end unit;",
    );
    assert!(
        matches!(&unit.declarations[0], Decl::Function(function) if function.pure && function.visibility == Visibility::Public)
    );
}

#[test]
fn pure_procedures_and_repeated_modifiers_are_rejected() {
    for declaration in [
        "pure procedure Action(); begin null; end procedure;",
        "type Action = pure procedure();",
        "pure pure function Value(): integer; begin return 1; end function;",
    ] {
        let (_, errors) = parse_with_errors(&format!(
            "program Main; {declaration} begin null; end program;"
        ));
        assert!(!errors.is_empty(), "{declaration}");
    }
    let (_, errors) = parse_with_errors(
        "program Main; begin const Action := pure procedure() begin null; end procedure; end program;",
    );
    assert!(!errors.is_empty());
}
