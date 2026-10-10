//! Distinct type conversions in debugger evaluation, including unit-scoped names.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use super::named_arguments::{evaluate, server};
use super::*;

#[test]
fn distinct_and_builtin_type_names_convert_scalar_values() {
    let mut server = server(
        "program DebugDistinct;
type UserId = distinct integer;
type Name = distinct string;
type Price = distinct real;
type Flag = distinct boolean;
begin end.",
    );
    for (expression, expected) in [
        ("integer(UserId(41)) + 1", "42"),
        ("USERID(7)", "7"),
        ("string(Name('Ada'))", "'Ada'"),
        ("real(Price(2.5))", "2.5"),
        ("boolean(Flag(true))", "true"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(
            response["body"]["result"], expected,
            "{expression}: {response}"
        );
    }
    for (expression, expected) in [
        ("UserId('x')", "requires a integer value, got string"),
        (
            "integer(Name('Ada'))",
            "requires a integer value, got string",
        ),
        ("UserId(1, 2)", "expects 1 argument, got 2"),
        ("Missing(1)", "not present in the executable catalog"),
    ] {
        let response = evaluate(&mut server, expression);
        assert_eq!(response["success"], false, "{expression}: {response}");
        assert!(
            response.to_string().contains(expected),
            "{expression}: {response}"
        );
    }
}

const UNIT: &str = "unit Ids;
public type UserId = distinct integer;
end unit;";

fn linked(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (unit, diagnostics) = fpas_parser::parse_compilation_unit(UNIT);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let fpas_parser::CompilationUnit::Unit(unit) = unit else {
        panic!("unit");
    };
    let mut unit = fpas_compiler::compile_unit_object(&unit, &[]).expect("unit object");
    unit.object.sources = vec!["ids.fpas".into()];
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let interfaces = [unit.interface];
    let mut root =
        fpas_compiler::compile_program_object_with_support(&program, &interfaces, &interfaces)
            .expect("root object");
    root.sources = vec!["consumer.fpas".into()];
    let unit = fpas_unit::object::decode_object(
        &fpas_unit::object::encode_object(&unit.object).expect("encode unit"),
    )
    .expect("decode unit");
    fpas_linker::link_objects(&[unit], &root).expect("link distinct types")
}

#[test]
fn imported_distinct_names_follow_the_source_import_form() {
    let wrap = |name: &str| DebugExpression::Call {
        callee: Box::new(DebugExpression::Name(name.into())),
        arguments: vec![DebugExpression::Integer(5)],
    };
    let mut aliased =
        DebugSession::new(linked("program Consumer; uses Ids as I; begin end.")).expect("session");
    assert_eq!(
        aliased
            .evaluate(&wrap("I.UserId"), None)
            .expect("aliased conversion")
            .value,
        "5"
    );
    for hidden in ["UserId", "Ids.UserId"] {
        assert_eq!(
            aliased
                .evaluate(&wrap(hidden), None)
                .expect_err("aliased import hides other names")
                .kind,
            DebugErrorKind::UnknownCallable,
            "{hidden}"
        );
    }
    let mut plain =
        DebugSession::new(linked("program Consumer; uses Ids; begin end.")).expect("session");
    for visible in ["UserId", "Ids.UserId"] {
        assert_eq!(
            plain
                .evaluate(&wrap(visible), None)
                .expect("plain import conversion")
                .value,
            "5",
            "{visible}"
        );
    }
    let mut missing = DebugSession::new(linked("program Consumer; begin end.")).expect("session");
    assert_eq!(
        missing
            .evaluate(&wrap("UserId"), None)
            .expect_err("missing uses")
            .kind,
        DebugErrorKind::UnknownCallable
    );
}
