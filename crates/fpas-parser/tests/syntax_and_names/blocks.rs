use fpas_parser::{CompilationUnit, Stmt, parse_compilation_unit};

fn accepted(source: &str) -> CompilationUnit {
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{source}\n{errors:#?}");
    unit
}

fn rejected(source: &str, hint: &str) {
    let (_, errors) = parse_compilation_unit(source);
    assert!(!errors.is_empty(), "accepted invalid source: {source}");
    assert!(
        errors
            .iter()
            .any(|error| format!("{error:?}").contains(hint)),
        "expected {hint:?}: {errors:#?}"
    );
}

#[test]
fn every_available_named_body_and_plain_block_parses() {
    let bodies = [
        "null;",
        "begin null; end;",
        "if true then null; elsif false then null; else null; end if;",
        "case 1 of when 1: null; when 2, 3: null; else null; end case;",
        "for I: integer := 1 to 2 do null; end for;",
        "for I: integer := 2 downto 1 do null; end for;",
        "for Item: integer in [1, 2] do null; end for;",
        "while false do null; end while;",
        "repeat null; until true;",
        "var F: function(): integer := function(): integer begin return 1; end function;",
        "Consume(function(): integer begin return 1; end function, procedure() begin null; end procedure);",
        "var P: Point := record X := 1; end record; P := P with X := 2; end with;",
    ];
    for body in bodies {
        accepted(&format!("program T; begin {body} end program;"));
    }
    accepted(
        "unit App.Model; uses Std.Str as Text; public type Point = record X: integer; end record; type Color = enum Red; Blue; end enum; public function F(): integer; procedure P(); begin null; end procedure; begin P(); return 1; end function; end unit;",
    );
    accepted(
        "program T; type R = record static function Create(): integer; begin return 1; end function; procedure Touch(Self: R); begin null; end procedure; end record; begin null; end program;",
    );
}

#[test]
fn branches_store_lists_and_elsif_in_source_order() {
    let CompilationUnit::Program(program) = accepted(
        "program T; begin if true then null; null; elsif false then null; elsif true then null; else null; end if; end program;",
    ) else {
        panic!("expected program")
    };
    let Stmt::If {
        then_branch,
        elsif_branches,
        else_branch,
        ..
    } = &program.body[0]
    else {
        panic!("expected if")
    };
    assert!(matches!(then_branch.as_ref(), Stmt::StatementList(body, _) if body.len() == 2));
    assert_eq!(elsif_branches.len(), 2);
    assert!(else_branch.is_some());
}

#[test]
fn else_if_owns_its_own_closer_and_explicit_blocks_remain_explicit() {
    accepted(
        "program T; begin if true then begin null; end; else if false then null; end if; end if; end program;",
    );
    rejected(
        "program T; begin if true then null; else if false then null; end if; end program;",
        "use `elsif`",
    );
    rejected(
        "program T; begin if true then begin null; end if; end program;",
        "plain lexical block",
    );
}

#[test]
fn terminators_are_required_before_every_body_boundary() {
    for body in [
        "null",
        "begin null end;",
        "if true then null else null; end if;",
        "if true then null elsif false then null; end if;",
        "if true then null end if;",
        "case 1 of when 1: null when 2: null; end case;",
        "case 1 of when 1: null end case;",
        "repeat null until true;",
        "while false do null end while;",
        "for I: integer := 1 to 2 do null end for;",
    ] {
        rejected(
            &format!("program T; begin {body} end program;"),
            "Every statement ends",
        );
    }
}

#[test]
fn empty_and_separator_only_bodies_require_null() {
    for body in [
        "",
        "begin end;",
        "if true then end if;",
        "while false do end while;",
        "for I: integer := 1 to 2 do end for;",
        "repeat until true;",
        "case 1 of when 1: end case;",
    ] {
        rejected(
            &format!("program T; begin {body} end program;"),
            "Write `null;`",
        );
    }
    for body in [";", "null;;", "if true then null;; else null; end if;"] {
        rejected(
            &format!("program T; begin {body} end program;"),
            "empty statements",
        );
    }
}

#[test]
fn legacy_and_mismatched_closers_are_rejected() {
    for source in [
        "program T; begin null; end.",
        "program T; begin null; end;",
        "unit App;",
        "program T; procedure P(); begin null; end; begin null; end program;",
        "program T; type R = record X: integer; end; begin null; end program;",
        "program T; type E = enum A; end; begin null; end program;",
        "program T; begin while false do null; end for; end program;",
        "program T; begin Consume(procedure() begin null; end function); end program;",
    ] {
        rejected(source, "Expected `end");
    }
    rejected(
        "program T; begin case 1 of 1: null; end case; end program;",
        "start with `when`",
    );
}

#[test]
fn expression_closers_leave_argument_and_statement_delimiters_to_the_caller() {
    accepted(
        "program T; begin Consume(function(): integer begin return 1; end function, 2); Consume(procedure() begin null; end procedure); end program;",
    );
    rejected(
        "program T; begin Consume(procedure() begin null; end procedure;); end program;",
        "Expected `)`",
    );
    accepted(
        "program T; begin Consume(record X := record Y := 2; end record; end record with X := record Y := 3; end record; end with, 4); end program;",
    );
}

#[test]
fn declaration_keywords_and_aliases_are_individual() {
    let CompilationUnit::Unit(unit) = accepted(
        "unit App; uses Std.Str as Text; uses Std.Console as Console; public const A: integer := 1; const B: integer := 2; public var C: integer := 3; var D: integer := 4; type E = integer; type F = string; end unit;",
    ) else {
        panic!("expected unit")
    };
    assert_eq!(unit.uses[0].alias, "Text");
    assert_eq!(unit.uses[1].parts, ["Std", "Console"]);
    assert_eq!(unit.declarations.len(), 6);
    for declarations in [
        "const A: integer := 1; B: integer := 2;",
        "var A: integer := 1; B: integer := 2;",
        "type A = integer; B = string;",
    ] {
        rejected(
            &format!("unit App; {declarations} end unit;"),
            "Each declaration requires",
        );
    }
    rejected(
        "program T; uses Std.Console; begin null; end program;",
        "explicit alias",
    );
    rejected(
        "program T; uses Std.Console as Console, Std.Str as Text; begin null; end program;",
        "comma-separated unit list",
    );
}

#[test]
fn comments_case_and_keyword_prefixes_do_not_change_boundaries() {
    accepted(
        "PrOgRaM T; uses Std.Str aS Text; BeGiN // end if;\n var nullValue: integer := 1; IF true THEN Text.Length('end program; null;'); ElSiF false THEN null; ELSE null; END IF; EnD PrOgRaM;",
    );
}

#[test]
fn truncated_and_wrong_outer_closers_terminate_recovery() {
    for source in [
        "program T; begin if true then",
        "program T; begin if true then null; end program;",
        "program T; begin while false do if true then null; end while; end program;",
        "program T; begin Consume(record X := 1;",
        "unit App; type R = record",
    ] {
        let (_, errors) = parse_compilation_unit(source);
        assert!(!errors.is_empty());
        assert!(errors.len() < 20, "unbounded cascade: {errors:#?}");
    }
}
