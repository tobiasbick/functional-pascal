//! Ada context clauses, main procedures, packages, tasks, exception handlers and `raise`: one
//! diagnostic each, recovery, and no false positives.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

use super::foreign_habit_assertions::{
    LATER_ERROR, assert_one_habit, compilation_unit_errors, parser_errors,
};
use crate::ast::CompilationUnit;
use crate::tests::{parse_compilation_unit_with_errors, parse_ok};
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;

#[test]
fn ada_context_clauses_and_main_procedure_are_reported_once_each() {
    let source = "with Ada.Text_IO; use Ada.Text_IO;\n\nprocedure Main is\nbegin\n  Put_Line('Hi');\nend Main;";
    let errors = compilation_unit_errors(source);
    let codes = errors.iter().map(|error| error.code).collect::<Vec<_>>();
    assert_eq!(
        codes,
        [PARSE_FOREIGN_SYNTAX, PARSE_FOREIGN_SYNTAX],
        "{errors:#?}"
    );
    assert_eq!(
        errors[0].found.as_deref(),
        Some("with Ada.Text_IO; use Ada.Text_IO;")
    );
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("uses Std.Console as Console;")
    );
    assert_eq!(
        errors[1].expected.as_deref(),
        Some("program Main; … begin … end.")
    );
    assert!(!errors[1].fixes.is_empty(), "{errors:#?}");

    let (unit, _) = parse_compilation_unit_with_errors(source);
    let CompilationUnit::Program(program) = unit else {
        panic!("Ada main procedure recovers as a program");
    };
    assert_eq!(program.name, "Main");
    assert_eq!(program.body.len(), 1);
}

#[test]
fn context_clauses_map_ada_library_units_and_keep_user_units() {
    let errors = compilation_unit_errors(
        "with Ada.Strings.Unbounded, Ada.Numerics.Float_Random;\nwith Geometry;\n\nprogram Main;\nbegin\nend.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("uses Std.Math.Random as Random, Geometry as Geometry;")
    );

    let errors =
        compilation_unit_errors("with Ada.Containers.Vectors;\n\nprogram Main;\nbegin\nend.");
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("no `uses` clause: strings, arrays and dictionaries are built-in types")
    );
}

const BODY: &str =
    "public function Twice(X: integer): integer;\nbegin\n  return X * 2;\nend function;";

#[test]
fn ada_package_body_is_parsed_as_a_unit() {
    let source = format!("package body Geometry is\n\n{BODY}\n\nend Geometry;");
    let (unit, _) = parse_compilation_unit_with_errors(&source);
    let errors = compilation_unit_errors(&source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].found.as_deref(), Some("package body Geometry is"));
    assert_eq!(
        errors[0].expected.as_deref(),
        Some("unit Geometry; … end unit;")
    );
    let CompilationUnit::Unit(unit) = unit else {
        panic!("package recovers as a unit");
    };
    assert_eq!(unit.name.parts, ["Geometry"]);
    assert_eq!(unit.declarations.len(), 1);
}

#[test]
fn ada_specification_before_its_body_is_skipped_and_reported_once() {
    let source = format!(
        "package Geometry is\n  function Twice (X : integer) return integer;\nend Geometry;\n\npackage body Geometry is\n\n{BODY}\n\nend Geometry;"
    );
    let errors = compilation_unit_errors(&source);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let (CompilationUnit::Unit(unit), _) = parse_compilation_unit_with_errors(&source) else {
        panic!("package recovers as a unit");
    };
    assert_eq!(unit.declarations.len(), 1);

    let broken = source.replace("return X * 2;", "Foo(1;");
    let errors = compilation_unit_errors(&broken);
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert_eq!(errors[1].span.expect("source span").line(), 9);
}

#[test]
fn ada_task_declarations_are_reported_once() {
    let replacement = "procedure Work(); … go Work() inside scope … end scope;";
    for (declaration, found) in [
        ("task Worker;", "task Worker;"),
        (
            "task type Worker is\n  entry Start;\nend Worker;",
            "task type Worker is",
        ),
        (
            "task body Worker is\nbegin\n  loop\n    null;\n  end loop;\nend Worker;",
            "task body Worker is",
        ),
    ] {
        assert_one_habit(
            declaration,
            PARSE_FOREIGN_SYNTAX,
            "Ada task declarations",
            found,
            replacement,
        );
    }
}

#[test]
fn ada_exception_handlers_are_skipped_to_the_end_of_their_block() {
    let replacement = "a `Result of (T, E)` handled with `case` or propagated with `try Expr`";
    for handler in [
        "exception\n  when others =>\n    null;",
        "exception\n  when Constraint_Error =>\n    if True then\n      null;\n    end if;",
    ] {
        assert_one_habit(
            &format!("procedure Q();\nbegin\n  null;\n{handler}\nend procedure;"),
            PARSE_FOREIGN_SYNTAX,
            "Ada exception handlers",
            &handler
                .split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" "),
            replacement,
        );
    }
}

#[test]
fn raise_statements_name_both_failure_forms() {
    assert_one_habit(
        "procedure Q();\nbegin\n  raise Constraint_Error;\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "`raise`",
        "raise Constraint_Error",
        "return Error('…'); or panic('…');",
    );
    assert_one_habit(
        "procedure Q();\nbegin\n  raise Exception.Create('bad input');\nend procedure;",
        PARSE_FOREIGN_SYNTAX,
        "`raise`",
        "raise Exception.Create('bad input')",
        "return Error('bad input'); or panic('bad input');",
    );
}

#[test]
fn identifiers_spelled_like_ada_keywords_are_not_reported() {
    parse_ok(&format!(
        "program P;

procedure Raise(Code: integer);
begin
  null;
end procedure;

procedure Q();
begin
  const Exception: integer := 1;
  const Package: integer := Exception;
  Raise(Package);
end procedure;

{}",
        LATER_ERROR.replace("Foo(1;", "Q();")
    ));
    assert!(parser_errors("program P;\nbegin\nend.").is_empty());
}
