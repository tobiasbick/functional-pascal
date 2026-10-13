use super::*;

#[test]
fn clauses_retain_source_text_order_and_imported_source_ids() {
    let source = "unit Api; public function Check(X: integer): integer;\nReQuIrEs X > 0;\nrequires X < 10;\nbegin return X; end function; end unit;";
    let (tokens, _, errors) = fpas_lexer::lex_with_source_id(source, 17);
    assert!(errors.is_empty());
    let (parsed, errors) = crate::parse_tokens_compilation_unit(tokens, source);
    assert!(errors.is_empty(), "{errors:?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("unit");
    };
    let Decl::Function(function) = &unit.declarations[0] else {
        panic!("function");
    };
    assert_eq!(
        function
            .requires
            .iter()
            .map(|clause| clause.text.as_str())
            .collect::<Vec<_>>(),
        ["ReQuIrEs X > 0;", "requires X < 10;"]
    );
    assert_eq!(function.requires[0].span.source_id, 17);
    assert_eq!(function.requires[0].span.line, 2);
}

#[test]
fn missing_clause_terminator_recovers_at_the_next_clause_and_body() {
    let (program, errors) = parse_with_errors(
        "program T; procedure Check(X: integer); requires X > 0 requires X < 10; begin null; end procedure; begin Check(1); end.",
    );
    assert!(!errors.is_empty());
    let Decl::Procedure(procedure) = &program.declarations[0] else {
        panic!("procedure");
    };
    assert_eq!(procedure.requires.len(), 2);
    assert_eq!(procedure.requires[1].text, "requires X < 10;");
}

#[test]
fn contract_keywords_are_reserved_in_every_identifier_position() {
    for name in ["requires", "EnSuReS"] {
        for source in [
            format!("program T; const {name}: integer := 1; begin end."),
            format!(
                "program T; procedure P({name}: integer); begin null; end procedure; begin end."
            ),
            format!("program T; type R = record {name}: integer; end record; begin end."),
            format!("program T; begin const X: integer := Value.{name}; end."),
        ] {
            let (_, errors) = parse_with_errors(&source);
            assert!(
                errors.iter().any(|error| error.as_diagnostic().code
                    == fpas_diagnostics::codes::PARSE_EXPECTED_IDENTIFIER),
                "{source}: {errors:?}"
            );
        }
    }
}

#[test]
fn postcondition_binders_require_a_colon_and_grouped_names_are_conditions() {
    let (program, errors) = parse_with_errors(
        "program T; function F(Flag: boolean): boolean;
         ensures (Flag); ensures (Returned): Returned = Flag;
         ensures (Flag and true); ensures (Flag) or false;
         begin return Flag; end function; begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let Decl::Function(function) = &program.declarations[0] else {
        panic!("function");
    };
    assert!(function.ensures[0].result_name.is_none());
    assert!(matches!(function.ensures[0].condition, Expr::Paren(..)));
    assert_eq!(
        function.ensures[1]
            .result_name
            .as_ref()
            .map(|(name, _)| name.as_str()),
        Some("Returned")
    );
    assert!(function.ensures[2].result_name.is_none());
    assert!(function.ensures[3].result_name.is_none());
    for condition in [
        "Returned > 0",
        "Some(1) is Some(_)",
        "Ok(1) is Ok(_)",
        "Error(1) is Error(_)",
        "None is None",
        "array.Fill(1, 0).Length() > 0",
        "string.Chr(65).Length() = 1",
    ] {
        let (_, errors) = parse_with_errors(&format!(
            "program T; function F(): integer; ensures (Returned) {condition}; begin return 1; end function; begin end."
        ));
        assert!(!errors.is_empty(), "legacy return binder was accepted");
        assert!(
            errors.iter().any(|error| error
                .as_diagnostic()
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains("ensures (Returned):"))),
            "{condition}: {errors:#?}"
        );
    }
}

#[test]
fn postconditions_parse_names_parenthesized_conditions_and_source_text() {
    let source = "program T; function F(X: integer): integer;
        requires X >= 0; EnSuReS (Value): Value > X;
        ensures (X >= 0); ensures (Other): Other > 0;
        begin return X + 1; end function; begin end.";
    let (program, errors) = parse_with_errors(source);
    assert!(errors.is_empty(), "{errors:?}");
    let Decl::Function(function) = &program.declarations[0] else {
        panic!("function");
    };
    assert_eq!(function.ensures[0].text, "EnSuReS (Value): Value > X;");
    assert_eq!(
        function.ensures[0]
            .result_name
            .as_ref()
            .map(|(name, _)| name.as_str()),
        Some("Value")
    );
    assert!(function.ensures[1].result_name.is_none());
    assert_eq!(
        function.ensures[2]
            .result_name
            .as_ref()
            .map(|(name, _)| name.as_str()),
        Some("Other")
    );
    let (program, errors) = parse_with_errors(
        "program T; procedure P(); ensures true ensures false; begin null; end procedure; begin end.",
    );
    assert!(!errors.is_empty());
    let Decl::Procedure(procedure) = &program.declarations[0] else {
        panic!("procedure");
    };
    assert_eq!(procedure.ensures.len(), 2);
    let (_, errors) = parse_with_errors(
        "program T; procedure P(); ensures true; requires true; begin null; end procedure; begin end.",
    );
    assert!(!errors.is_empty(), "entry clauses precede exit clauses");
}
