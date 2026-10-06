//! AP13.4 formatting, explicit scopes, branch ownership, and comment preservation.

mod common;

fn format(source: &str) -> String {
    common::assert_round_trip("control blocks", source);
    let (unit, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:#?}");
    fpas_fmt::format_source(source, &unit).expect("source and AST agree")
}

#[test]
fn control_lists_indent_without_inserting_compound_blocks() {
    let formatted = format(
        "program T; begin if true then A(); B(); elsif false then null; else C(); end if;
        for I: integer := 1 to 3 do A(I); B(); end for;
        while false do null; end while; repeat null; until true; end.",
    );
    assert!(formatted.contains("if true then\n    A();\n    B();\n  elsif false then\n    null;\n  else\n    C();\n  end if;"), "{formatted}");
    assert_eq!(formatted.matches("begin").count(), 1, "{formatted}");
    assert!(formatted.contains("end for;"));
    assert!(formatted.contains("end while;"));
}

#[test]
fn explicit_blocks_and_nested_else_if_keep_their_ownership() {
    let formatted = format(
        "program T; begin if true then begin A(); end; B(); else if false then C(); end if; end if; end.",
    );
    assert_eq!(formatted.matches("end if;").count(), 2);
    assert!(!formatted.contains("elsif"));
    assert!(formatted.contains("else\n    if false then"), "{formatted}");
    assert!(
        formatted.contains("then\n    begin\n      A();\n    end;\n\n    B();"),
        "{formatted}"
    );
}

#[test]
fn comments_at_clauses_and_named_endings_are_preserved_once() {
    let formatted = format(
        "program T; begin
        if true then // then header
            null; // then statement
        // before elsif
        elsif false then null;
        // before else
        else null;
        // before if end
        end // inside if end
        if; // after if end
        for I: integer := 1 to 2 do // for header
            null;
        // before for end
        end for; // after for end
        while false do null;
        // before while end
        end while; // after while end
        end.",
    );
    for text in [
        "then header",
        "then statement",
        "before elsif",
        "before else",
        "before if end",
        "inside if end",
        "after if end",
        "for header",
        "before for end",
        "after for end",
        "before while end",
        "after while end",
    ] {
        assert_eq!(
            formatted.matches(&format!("// {text}")).count(),
            1,
            "{formatted}"
        );
    }
    assert!(formatted.contains("// before else\n  else"), "{formatted}");
    assert!(formatted.contains("end if; // after if end"), "{formatted}");
    assert!(
        formatted.contains("end for; // after for end"),
        "{formatted}"
    );
    assert!(
        formatted.contains("end while; // after while end"),
        "{formatted}"
    );
}
