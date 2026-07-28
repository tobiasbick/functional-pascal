use super::*;
use fpas_diagnostics::codes::PARSE_INVALID_STATIC_PLACEMENT;

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
fn unit_record_routines_preserve_per_member_visibility() {
    let unit = parse_unit_ok(
        "unit Demo.Types; \
         type Counter = record \
           function Hidden(Self: Counter): integer; begin return 1 end; \
           public procedure Reset(Self: Counter); begin end; \
           static function CreateHidden(): Counter; begin return record end end; \
           public static procedure Clear(); begin end; \
         end;",
    );
    let Decl::TypeDef(type_def) = &unit.declarations[0] else {
        panic!("expected TypeDef");
    };
    let TypeBody::Record(record) = &type_def.body else {
        panic!("expected Record");
    };

    assert_eq!(record.methods[0].visibility(), Visibility::Private);
    assert_eq!(record.methods[1].visibility(), Visibility::Public);
    assert_eq!(record.methods[2].visibility(), Visibility::Private);
    assert_eq!(record.methods[3].visibility(), Visibility::Public);
}

#[test]
fn public_record_routine_is_rejected_in_program_files() {
    let (_, errors) = parse_with_errors(
        "program T; \
         type Counter = record \
           public function Hidden(Self: Counter): integer; begin return 1 end; \
         end; \
         begin end.",
    );

    assert!(errors.iter().any(|diagnostic| {
        matches!(
            diagnostic,
            ParseDiagnostic::Parser(error) if error.code == fpas_diagnostics::codes::PARSE_INVALID_VISIBILITY
        )
    }));
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

#[test]
fn record_with_static_function() {
    let p = parse_ok(
        "program T; type Point = record X: integer; Y: integer; \
         static function Create(X: integer; Y: integer): Point; \
         begin return record X := X; Y := Y; end end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.fields.len(), 2);
                assert_eq!(r.methods.len(), 1);
                match &r.methods[0] {
                    RecordMethod::StaticFunction(f) => {
                        assert_eq!(f.name, "Create");
                        assert_eq!(f.params.len(), 2);
                    }
                    other => panic!("expected StaticFunction, got {other:?}"),
                }
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_static_and_instance_methods_together() {
    let p = parse_ok(
        "program T; type Point = record X: integer; Y: integer; \
         static function Origin(): Point; \
         begin return record X := 0; Y := 0; end end; \
         function Sum(Self: Point): integer; begin return Self.X + Self.Y end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => {
                assert_eq!(r.methods.len(), 2);
                assert!(matches!(&r.methods[0], RecordMethod::StaticFunction(_)));
                assert!(matches!(&r.methods[1], RecordMethod::Function(_)));
            }
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn record_with_static_procedure() {
    let p = parse_ok(
        "program T; type Point = record X: integer; \
         static procedure Reset(X: integer); begin end; \
         end; begin end.",
    );
    match &p.declarations[0] {
        Decl::TypeDef(td) => match &td.body {
            TypeBody::Record(r) => match &r.methods[0] {
                RecordMethod::StaticProcedure(procedure) => {
                    assert_eq!(procedure.name, "Reset");
                    assert_eq!(procedure.params.len(), 1);
                }
                other => panic!("expected StaticProcedure, got {other:?}"),
            },
            _ => panic!("expected Record"),
        },
        _ => panic!("expected TypeDef"),
    }
}

#[test]
fn static_at_program_level_is_rejected() {
    let (_p, errs) = parse_with_errors(
        "program T; static function Foo(): integer; begin return 1 end; begin end.",
    );
    let parse_err = errs.iter().find_map(ParseDiagnostic::as_parser_error);
    let d = parse_err.expect("expected parser diagnostic");
    assert_eq!(d.code, PARSE_INVALID_STATIC_PLACEMENT);
    assert!(
        d.message
            .contains("only valid on a function or procedure declared inside a record"),
        "message: {}",
        d.message
    );
}

#[test]
fn static_procedure_at_program_level_is_rejected() {
    let (_p, errs) =
        parse_with_errors("program T; static procedure Reset(); begin end; begin end.");
    let parse_err = errs.iter().find_map(ParseDiagnostic::as_parser_error);
    let d = parse_err.expect("expected parser diagnostic");
    assert_eq!(d.code, PARSE_INVALID_STATIC_PLACEMENT);
    assert!(
        d.message
            .contains("only valid on a function or procedure declared inside a record"),
        "message: {}",
        d.message
    );
}

#[test]
fn static_without_function_in_record_is_rejected() {
    let (_p, errs) =
        parse_with_errors("program T; type Point = record X: integer; static; end; begin end.");
    let parse_err = errs.iter().find_map(ParseDiagnostic::as_parser_error);
    let d = parse_err.expect("expected parser diagnostic");
    assert_eq!(d.code, PARSE_INVALID_STATIC_PLACEMENT);
}
