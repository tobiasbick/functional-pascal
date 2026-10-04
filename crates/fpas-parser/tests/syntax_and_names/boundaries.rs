use fpas_parser::parse_compilation_unit;

// The suffix supplies the caller's delimiter for expression-owned closers.
struct BlockCase {
    name: &'static str,
    prefix: &'static str,
    closer: &'static str,
    suffix: &'static str,
}

const BLOCKS: &[BlockCase] = &[
    BlockCase {
        name: "program",
        prefix: "program P; begin null; ",
        closer: "end program",
        suffix: ";",
    },
    BlockCase {
        name: "unit",
        prefix: "unit Library; const Value: integer := 1; ",
        closer: "end unit",
        suffix: ";",
    },
    BlockCase {
        name: "function",
        prefix: "program P; function F(): integer; begin return 1; ",
        closer: "end function",
        suffix: "; begin null; end program;",
    },
    BlockCase {
        name: "procedure",
        prefix: "program P; procedure P(); begin null; ",
        closer: "end procedure",
        suffix: "; begin null; end program;",
    },
    BlockCase {
        name: "anonymous function",
        prefix: "program P; begin Consume(function(): integer begin return 1; ",
        closer: "end function",
        suffix: ", 2); end program;",
    },
    BlockCase {
        name: "anonymous procedure",
        prefix: "program P; begin Consume(procedure() begin null; ",
        closer: "end procedure",
        suffix: "); end program;",
    },
    BlockCase {
        name: "plain block",
        prefix: "program P; begin begin null; ",
        closer: "end",
        suffix: "; end program;",
    },
    BlockCase {
        name: "record declaration",
        prefix: "program P; type R = record X: integer; ",
        closer: "end record",
        suffix: "; begin null; end program;",
    },
    BlockCase {
        name: "enum declaration",
        prefix: "program P; type E = enum A; ",
        closer: "end enum",
        suffix: "; begin null; end program;",
    },
    BlockCase {
        name: "if",
        prefix: "program P; begin if true then null; ",
        closer: "end if",
        suffix: "; end program;",
    },
    BlockCase {
        name: "case",
        prefix: "program P; begin case 1 of when 1: null; ",
        closer: "end case",
        suffix: "; end program;",
    },
    BlockCase {
        name: "counted for",
        prefix: "program P; begin for I: integer := 1 to 2 do null; ",
        closer: "end for",
        suffix: "; end program;",
    },
    BlockCase {
        name: "collection for",
        prefix: "program P; begin for I: integer in [1] do null; ",
        closer: "end for",
        suffix: "; end program;",
    },
    BlockCase {
        name: "while",
        prefix: "program P; begin while false do null; ",
        closer: "end while",
        suffix: "; end program;",
    },
    BlockCase {
        name: "record update",
        prefix: "program P; begin Consume(Value with X := 1; ",
        closer: "end with",
        suffix: ", 2); end program;",
    },
];

fn assert_rejected(name: &str, source: &str, message: &str) {
    let (_, errors) = parse_compilation_unit(source);
    assert!(
        errors.iter().any(|error| error
            .as_parser_error()
            .is_some_and(|error| error.message.contains(message))),
        "{name}: {source}\n{errors:#?}"
    );
    assert!(errors.len() < 30, "{name}: recovery cascade: {errors:#?}");
}

#[test]
fn every_block_accepts_its_closer_and_rejects_missing_or_mismatched_closers() {
    for case in BLOCKS {
        let source = format!("{}{}{}", case.prefix, case.closer, case.suffix);
        let (_, errors) = parse_compilation_unit(&source);
        assert!(errors.is_empty(), "{}: {errors:#?}", case.name);

        let expected = if case.closer == "end" {
            "plain lexical block".to_owned()
        } else {
            format!("Expected `{}`", case.closer)
        };
        for wrong in [
            "",
            "end",
            "end program",
            "end unit",
            "end function",
            "end procedure",
            "end record",
            "end enum",
            "end if",
            "end case",
            "end for",
            "end while",
            "end with",
        ] {
            if wrong != case.closer {
                assert_rejected(
                    case.name,
                    &format!("{}{wrong}{}", case.prefix, case.suffix),
                    &expected,
                );
            }
        }
    }
}

#[test]
fn final_body_and_declaration_terminators_are_mandatory() {
    for case in BLOCKS {
        let prefix = case.prefix.trim_end().strip_suffix(';').unwrap();
        assert_rejected(
            case.name,
            &format!("{prefix} {}{}", case.closer, case.suffix),
            "Expected `;`",
        );
    }
    for body in [
        "if true then null; elsif false then null else null; end if;",
        "if true then null; else null end if;",
        "case 1 of when 1: null else null; end case;",
        "case 1 of when 1: null; else null end case;",
        "repeat null until true;",
    ] {
        assert_rejected(
            "branch or repeat boundary",
            &format!("program P; begin {body} end program;"),
            "Expected `;`",
        );
    }
}

#[test]
fn statement_and_declaration_closers_need_their_own_final_terminator() {
    for case in BLOCKS.iter().filter(|case| case.suffix.starts_with(';')) {
        assert_rejected(
            case.name,
            &format!("{}{}{}", case.prefix, case.closer, &case.suffix[1..]),
            "Expected `;`",
        );
    }
    assert_rejected(
        "repeat condition",
        "program P; begin repeat null; until true end program;",
        "Expected `;`",
    );
}

#[test]
fn extra_terminators_do_not_create_empty_bodies_or_expression_statements() {
    for case in BLOCKS {
        assert_rejected(
            case.name,
            &format!("{};{}{}", case.prefix, case.closer, case.suffix),
            if matches!(
                case.name,
                "unit" | "record declaration" | "enum declaration" | "record update"
            ) {
                "Expected"
            } else {
                "empty statements"
            },
        );
    }
    for case in BLOCKS.iter().filter(|case| !case.suffix.starts_with(';')) {
        assert_rejected(
            case.name,
            &format!("{}{};{}", case.prefix, case.closer, case.suffix),
            "Expected `)`",
        );
    }
}

#[test]
fn empty_statement_bodies_require_null_in_every_branch_and_callable() {
    for source in [
        "program P; function F(): integer; begin end function; begin null; end program;",
        "program P; procedure P(); begin end procedure; begin null; end program;",
        "program P; begin Consume(function(): integer begin end function); end program;",
        "program P; begin Consume(procedure() begin end procedure); end program;",
        "program P; begin if true then null; elsif false then else null; end if; end program;",
        "program P; begin if true then null; else end if; end program;",
        "program P; begin case 1 of when 1: null; else end case; end program;",
        "program P; begin for I: integer in [1] do end for; end program;",
    ] {
        assert_rejected("empty statement body", source, "Empty statement bodies");
    }
    for source in [
        "unit Empty; end unit;",
        "program P; type Empty = record end record; begin null; end program;",
        "program P; begin Consume(Empty()); end program;",
    ] {
        assert!(parse_compilation_unit(source).1.is_empty(), "{source}");
    }
}

#[test]
fn empty_enum_and_record_update_diagnostics_teach_valid_named_closers() {
    for (invalid, valid, closer) in [
        (
            "program P; type Color = enum end enum; begin null; end program;",
            "program P; type Color = enum Red; end enum; begin null; end program;",
            "end enum",
        ),
        (
            "program P; begin Consume(Value with end with); end program;",
            "program P; begin Consume(Value with X := 1; end with); end program;",
            "end with",
        ),
    ] {
        let (_, errors) = parse_compilation_unit(invalid);
        assert!(
            errors
                .iter()
                .any(|error| error.as_parser_error().is_some_and(|error| error
                    .help
                    .as_deref()
                    .is_some_and(|help| help.contains(closer)))),
            "{invalid}: {errors:#?}"
        );
        let (_, errors) = parse_compilation_unit(valid);
        assert!(errors.is_empty(), "{valid}: {errors:#?}");
    }
}

#[test]
fn repeat_ends_at_until_and_preserves_the_following_statement() {
    let source =
        "program P; begin repeat if true then null; end if; until true; null; end program;";
    let (unit, errors) = parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    let fpas_parser::CompilationUnit::Program(program) = unit else {
        panic!("expected program");
    };
    assert_eq!(program.body.len(), 2);
    for body in [
        "repeat null;",
        "repeat null; end while;",
        "repeat null; until true; end repeat;",
    ] {
        assert!(
            !parse_compilation_unit(&format!("program P; begin {body} end program;"))
                .1
                .is_empty(),
            "{body}"
        );
    }
}

#[test]
fn grouped_names_teach_one_declaration_per_name() {
    for (name, source, example) in [
        (
            "variable",
            "program P; var A, B: integer := 1; begin null; end program;",
            "`var A: integer := 0; var B: integer := 0;`",
        ),
        (
            "constant",
            "program P; const A, B: integer := 1; begin null; end program;",
            "`var A: integer := 0; var B: integer := 0;`",
        ),
        (
            "local variable",
            "program P; begin var A, B, C: integer := 1; end program;",
            "`var A: integer := 0; var B: integer := 0; var C: integer := 0;`",
        ),
        (
            "parameter",
            "program P; procedure Q(A, B: integer); begin null; end procedure; begin null; end program;",
            "`A: integer; B: integer`",
        ),
    ] {
        let (_, errors) = parse_compilation_unit(source);
        let errors: Vec<_> = errors
            .iter()
            .filter_map(|error| error.as_parser_error())
            .collect();
        assert_eq!(errors.len(), 1, "{name}: {errors:#?}");
        assert_eq!(
            errors[0].message, "Each declaration names exactly one binding",
            "{name}"
        );
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains(example)),
            "{name}: {errors:#?}"
        );
    }
}
