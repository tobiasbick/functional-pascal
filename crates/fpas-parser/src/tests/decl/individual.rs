//! Individual declaration keywords, visibility, and group recovery.

use super::*;
use fpas_diagnostics::codes::PARSE_MISSING_DECLARATION_KEYWORD;

#[test]
fn each_declaration_has_its_own_keyword_and_visibility() {
    let unit = parse_unit_ok(
        "unit U;
         public type First = integer; type Hidden = integer; public type Last = string;
         public const A: integer := 1; const B: integer := 2; public const C: integer := 3;
         public var D: integer := A; var E: integer := B; public var F: integer := C;
         public var G: integer := 0; var H: integer := 0;
         public var I: integer := 0; end unit;",
    );
    assert_eq!(unit.declarations.len(), 12);
    for declarations in unit.declarations.chunks(3) {
        assert_eq!(declarations[0].visibility(), Visibility::Public);
        assert_eq!(declarations[1].visibility(), Visibility::Private);
        assert_eq!(declarations[2].visibility(), Visibility::Public);
    }
    assert!(matches!(unit.declarations[9], Decl::Var(_)));
    assert!(matches!(unit.declarations[10], Decl::Var(_)));
    assert!(matches!(unit.declarations[11], Decl::Var(_)));
}

#[test]
fn removed_groups_report_each_missing_keyword_at_the_name() {
    for (first, second, prefix) in [
        ("type A = integer;", "B = string;", "type"),
        ("const A: integer := 1;", "B: integer := 2;", "const"),
        ("var A: integer := 1;", "B: integer := 2;", "var"),
        ("var A: integer := 1;", "B: integer := 2;", "var"),
    ] {
        for public in [false, true] {
            let modifier = if public { "public " } else { "" };
            let source = format!("unit U; {modifier}{first} {second} end unit;");
            let (unit, errors) = parse_compilation_unit_with_errors(&source);
            assert_eq!(errors.len(), 1, "{source}: {errors:?}");
            let error = errors[0].as_diagnostic();
            assert_eq!(error.code, PARSE_MISSING_DECLARATION_KEYWORD);
            assert_eq!(
                error.span.as_ref().unwrap().offset(),
                source.find(second).unwrap()
            );
            assert!(
                error
                    .help
                    .as_ref()
                    .unwrap()
                    .contains(&format!("{modifier}{prefix} B"))
            );
            let CompilationUnit::Unit(unit) = unit else {
                panic!("expected unit");
            };
            assert_eq!(unit.declarations.len(), 2);
            assert_eq!(unit.declarations[1].visibility(), Visibility::Private);
        }
    }
}

#[test]
fn group_recovery_keeps_following_explicit_declarations() {
    let (program, errors) = parse_with_errors(
        "program T; type A = integer; B = string; C = boolean;
         const N: integer := 1; var V: A := N; begin end.",
    );
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(
        errors
            .iter()
            .all(|error| error.as_diagnostic().code == PARSE_MISSING_DECLARATION_KEYWORD)
    );
    assert_eq!(program.declarations.len(), 5);
    assert!(matches!(program.declarations[3], Decl::Const(_)));
    assert!(matches!(program.declarations[4], Decl::Var(_)));
}

#[test]
fn local_variable_groups_repeat_the_complete_keyword() {
    for prefix in ["var", "var"] {
        let source = format!(
            "program T; procedure P(); begin {prefix} A: integer := 1;
            B: integer := 2; {prefix} C: integer := 3; end procedure; begin P(); end."
        );
        let (program, errors) = parse_with_errors(&source);
        assert_eq!(errors.len(), 1, "{source}: {errors:?}");
        assert_eq!(
            errors[0].as_diagnostic().code,
            PARSE_MISSING_DECLARATION_KEYWORD
        );
        assert!(
            errors[0]
                .as_diagnostic()
                .help
                .as_ref()
                .unwrap()
                .contains(&format!("{prefix} B:"))
        );
        let Decl::Procedure(procedure) = &program.declarations[0] else {
            panic!("expected procedure");
        };
        let FuncBody::Block { stmts, .. } = &procedure.body;
        assert_eq!(stmts.len(), 3);
        assert_eq!(matches!(stmts[1], Stmt::Var(_)), prefix == "var");
    }
}

#[test]
fn fields_payloads_and_parameters_keep_their_own_syntax() {
    let program = parse_ok(
        "program T; type Pair = record Left: integer; Right: integer; end record;
         type Message = enum Data(Left: integer; Right: integer); Empty; end enum;
         procedure P(Left: integer; Right: integer); begin
           var A: integer := Left; var B: integer := Right;
           var C: integer := A; var D: integer := B;
         end procedure; begin end.",
    );
    assert_eq!(program.declarations.len(), 3);
}

#[test]
fn individual_keywords_do_not_add_local_types() {
    let source = "program T; procedure P(); begin type Local = integer; end procedure; begin end.";
    let (_, errors) = parse_with_errors(source);
    assert!(!errors.is_empty(), "{source}");
}
