//! Alias-only imports, whole-unit types, and named branch scopes.

use fpas_parser::{CompilationUnit, Program};

fn program(source: &str) -> Program {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:#?}");
    program
}

fn errors(source: &str) -> Vec<fpas_sema::SemaError> {
    fpas_sema::analyze(&program(source))
}

#[test]
fn aliases_are_case_insensitive_and_dispatch_to_canonical_units() {
    let ast = program(
        "program P; uses Std.Str as Text; uses Std.Math as Numbers; begin var S: string := tExT.Trim(' a '); var I: integer := NUMBERS.Abs(-1); end program;",
    );
    let metadata = fpas_sema::analyze_with_types(&ast);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert_eq!(metadata.import_aliases["text"], "Std.Str");
}

#[test]
fn imports_expose_only_the_declared_alias() {
    for source in [
        "program P; uses Std.Str as Text; begin var S: string := Trim(' a '); end program;",
        "program P; uses Std.Str as Text; begin var S: string := Std.Str.Trim(' a '); end program;",
        "program P; uses Std.Str as Text; begin var S: string := Str.Trim(' a '); end program;",
        "program P; uses Std.Str as Text; begin var S: string := (' a ').Trim(); end program;",
        "program P; uses Std.Json as Json; begin var V: JsonValue := Json.JsonValue.NullValue; end program;",
    ] {
        let errors = errors(source);
        assert!(!errors.is_empty(), "unexpectedly accepted {source}");
        assert!(
            errors.iter().any(|error| error
                .help
                .as_ref()
                .is_some_and(|help| help.contains("text.") || help.contains("json."))),
            "{errors:#?}"
        );
    }
}

#[test]
fn imports_do_not_steal_lexical_names() {
    assert!(errors("program P; uses Std.Str as Text; function Trim(S: string): string; begin return S; end function; begin var S: string := Trim('a'); end program;").is_empty());
}

#[test]
fn import_collisions_include_every_lexical_binding_kind() {
    for declaration in [
        "var text: integer := 1;",
        "const TEXT: integer := 1;",
        "type Text = integer;",
        "procedure TEXT(); begin null; end procedure;",
        "procedure F(Text: integer); begin null; end procedure;",
        "function F<Text>(X: Text): Text; begin return X; end function;",
    ] {
        let source =
            format!("program P; uses Std.Str as Text; {declaration} begin null; end program;");
        assert!(
            errors(&source)
                .iter()
                .any(|error| error.message.contains("import alias")),
            "{source}"
        );
    }
    for statement in [
        "var Text: integer := 1;",
        "begin var Text: integer := 1; end;",
        "for Text: integer := 0 to 1 do null; end for;",
        "for Text: integer in [1] do null; end for;",
        "var F: procedure(Text: integer) := procedure(Text: integer) begin null; end procedure;",
    ] {
        let source = format!("program P; uses Std.Str as Text; begin {statement} end program;");
        assert!(
            errors(&source)
                .iter()
                .any(|error| error.message.contains("import alias")),
            "{source}"
        );
    }
}

#[test]
fn duplicate_units_and_aliases_are_rejected_case_insensitively() {
    for imports in [
        "uses Std.Str as Text; uses std.str as Other;",
        "uses Std.Str as Text; uses Std.Math as TEXT;",
    ] {
        assert!(!errors(&format!("program P; {imports} begin null; end program;")).is_empty());
    }
}

#[test]
fn forward_types_work_in_signatures_fields_aliases_and_initializers() {
    for source in [
        "program P; function F(X: Later): Later; begin return X; end function; type Later = integer; begin var I: Later := F(1); end program;",
        "program P; type First = Later; var I: First := 1; type Later = integer; begin null; end program;",
        "program P; type Outer = record Item: Later; end record; type Later = integer; begin var V: Outer := record Item := 1; end record; end program;",
        "program P; type Node = record Children: array of Node; end record; begin var N: Node := record Children := []; end record; end program;",
        "program P; type A = B; type B = C; type C = integer; begin var I: A := 1; end program;",
    ] {
        let errors = errors(source);
        assert!(errors.is_empty(), "{source}\n{errors:#?}");
    }
}

#[test]
fn initializer_order_unknown_types_and_alias_cycles_are_rejected() {
    for source in [
        "program P; var A: integer := B; var B: integer := 1; begin null; end program;",
        "program P; const A: integer := B; const B: integer := 1; begin null; end program;",
        "program P; type A = Missing; begin null; end program;",
        "program P; type A = B; type B = A; begin null; end program;",
        "program P; type A = A; begin null; end program;",
        "program P; type R = record X: integer := Later; end record; const Later: integer := 1; begin null; end program;",
    ] {
        assert!(!errors(source).is_empty(), "unexpectedly accepted {source}");
    }
    assert!(errors("program P; const Earlier: integer := 1; type R = record X: integer := Earlier; end record; begin var V: R := record end record; end program;").is_empty());
}

#[test]
fn type_and_routine_collisions_are_independent_of_order() {
    for declarations in [
        "type Count = integer; procedure COUNT(); begin null; end procedure;",
        "procedure Count(); begin null; end procedure; type COUNT = integer;",
        "type Count = integer; type COUNT = real;",
    ] {
        assert!(
            !errors(&format!(
                "program P; {declarations} begin null; end program;"
            ))
            .is_empty()
        );
    }
}

#[test]
fn each_branch_has_a_distinct_local_scope() {
    assert!(errors("program P; begin if true then var X: integer := 1; elsif false then var X: string := 's'; else var X: boolean := false; end if; var X: integer := 2; end program;").is_empty());
    for statements in [
        "if true then var X: integer := 1; else var Y: integer := X; end if;",
        "if true then var X: integer := 1; elsif X = 1 then null; end if;",
        "if true then var X: integer := 1; end if; var Y: integer := X;",
        "if true then begin var X: integer := 1; end; var Y: integer := X; end if;",
    ] {
        assert!(!errors(&format!("program P; begin {statements} end program;")).is_empty());
    }
}

#[test]
fn source_unit_imports_keep_types_variants_and_private_members_distinct() {
    let (CompilationUnit::Unit(unit), parse_errors) = fpas_parser::parse_compilation_unit(
        "unit Library.Values; public type Number = integer; public type Color = enum Red; end enum; type Hidden = integer; public function Twice(X: Number): Number; begin return X * 2; end function; end unit;",
    ) else {
        panic!("expected unit");
    };
    assert!(parse_errors.is_empty(), "{parse_errors:#?}");
    let analysis = fpas_sema::analyze_unit(&unit, &[]).unwrap();
    assert!(
        analysis.metadata.errors.is_empty(),
        "{:#?}",
        analysis.metadata.errors
    );
    let interface = analysis.interface.unwrap();
    for (body, valid) in [
        (
            "var I: Values.Number := Values.Twice(2); var C: Values.Color := Values.Color.Red;",
            true,
        ),
        ("var I: Number := 1;", false),
        ("var I: Values.Hidden := 1;", false),
        ("var I: Library.Values.Number := 1;", false),
        ("var C: Values.Color := Red;", false),
    ] {
        let ast = program(&format!(
            "program P; uses Library.Values as Values; begin {body} end program;"
        ));
        let metadata =
            fpas_sema::analyze_program_with_interfaces(&ast, std::slice::from_ref(&interface))
                .unwrap();
        assert_eq!(
            metadata.errors.is_empty(),
            valid,
            "{body}\n{:#?}",
            metadata.errors
        );
    }
}

#[test]
fn case_arm_declarations_are_local_even_without_pattern_bindings() {
    assert!(errors("program P; begin var X: integer := 1; case 1 of when 0: var X: string := 'zero'; when 1: var X: boolean := true; else var X: real := 2.0; end case; var Y: integer := X; end program;").is_empty());
    for body in [
        "case 1 of when 1: var Hidden: integer := 1; else null; end case; var X: integer := Hidden;",
        "case 1 of when 1: var Hidden: integer := 1; when 2: var X: integer := Hidden; end case;",
        "case 1 of when 1: null; else var Hidden: integer := 1; end case; var X: integer := Hidden;",
        "case Some(1) of when Some(Value): null; else var Hidden: integer := 1; end case; var X: integer := Hidden;",
    ] {
        let diagnostics = errors(&format!("program P; begin {body} end program;"));
        assert!(
            diagnostics
                .iter()
                .any(|error| error.message.contains("Hidden")),
            "{diagnostics:#?}"
        );
    }
}

#[test]
fn an_import_alias_cannot_open_a_nested_unit_namespace() {
    let interfaces = [
        "unit Library.Root; public type Item = integer; end unit;",
        "unit Library.Root.Nested; public const Answer: integer := 42; end unit;",
        "unit Library.Root.Branch.Leaf; public const Answer: integer := 7; end unit;",
    ]
    .into_iter()
    .map(|source| {
        let (CompilationUnit::Unit(unit), errors) = fpas_parser::parse_compilation_unit(source)
        else {
            panic!("expected unit")
        };
        assert!(errors.is_empty(), "{errors:#?}");
        fpas_sema::analyze_unit(&unit, &[])
            .unwrap()
            .interface
            .unwrap()
    })
    .collect::<Vec<_>>();
    for (name, valid) in [
        ("Nested.Answer", true),
        ("Leaf.Answer", true),
        ("Root.Nested.Answer", false),
        ("Root.Branch.Leaf.Answer", false),
    ] {
        let ast = program(&format!(
            "program P; uses Library.Root as Root;
            uses Library.Root.Nested as Nested;
            uses Library.Root.Branch.Leaf as Leaf; begin var Value: integer := {name}; end program;"
        ));
        let metadata = fpas_sema::analyze_program_with_interfaces(&ast, &interfaces).unwrap();
        assert_eq!(metadata.errors.is_empty(), valid, "{:#?}", metadata.errors);
        if !valid {
            assert_eq!(
                metadata
                    .errors
                    .iter()
                    .filter(|error| {
                        error
                            .message
                            .contains("names `Library.Root`, not `Library.Root.")
                    })
                    .count(),
                1,
                "one diagnostic per source path"
            );
        }
    }
}
