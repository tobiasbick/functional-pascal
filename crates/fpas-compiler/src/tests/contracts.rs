//! Entry checks and diagnostics observed through compiled execution.
use super::{assert_succeeds, run_program};

mod postconditions;

#[test]
fn disabled_contracts_emit_no_conditions_branches_or_diagnostic_preparation() {
    use fpas_runtime::Opcode;
    let program = super::parse_ok(
        "program T;
        function Guard(Value: integer): integer;
        requires 10 div Value > 0;
        ensures (Returned): 20 div Value > Returned;
        begin return Value; end function;
        begin discard Guard(0); end.",
    );
    for contracts in [true, false] {
        let image =
            crate::compile_with_options(&program, crate::CompileOptions { contracts }).unwrap();
        let opcodes: Vec<_> = image
            .decoded()
            .iter()
            .map(|instruction| instruction.opcode())
            .collect();
        assert_eq!(opcodes.contains(&Opcode::Panic), contracts);
        assert_eq!(opcodes.contains(&Opcode::MakeDictionary), contracts);
        assert_eq!(
            opcodes.iter().any(|opcode| matches!(
                opcode,
                Opcode::BranchIfFalse
                    | Opcode::BranchIfTrue
                    | Opcode::BranchIfEqualInteger
                    | Opcode::BranchIfNotEqualInteger
                    | Opcode::BranchIfLessInteger
                    | Opcode::BranchIfGreaterInteger
                    | Opcode::BranchIfLessEqualInteger
                    | Opcode::BranchIfGreaterEqualInteger
            )),
            contracts
        );
        assert_eq!(
            opcodes
                .iter()
                .any(|opcode| matches!(opcode, Opcode::DivideInteger | Opcode::DivideIntegerImm)),
            contracts
        );
        let result = fpas_runtime::execution::Vm::new(image).run();
        assert_eq!(result.is_err(), contracts);
    }
}

#[test]
fn preconditions_do_not_supply_a_missing_function_return() {
    for condition in ["true", "false"] {
        let program = super::parse_ok(&format!(
            "program T; function Guard(): integer;
            requires {condition}; begin null; end function;
            begin const Value: integer := Guard(); end."
        ));
        assert!(
            crate::compile(&program).is_err(),
            "requires {condition} cannot supply a return"
        );
    }
}

#[test]
fn clauses_panic_in_order_before_body_and_render_all_parameter_values() {
    let error = run_program(
        "program T; type Box = record Value: integer; end record;
        procedure Guard(X: integer; Text: string; Values: array of integer; B: Box);
        requires X > 0; requires 10 div X > 0;
        begin panic('body was entered'); end procedure;
        begin Guard(0, '50% 😀', [1, 2], Box(Value := 3)); end.",
    )
    .expect_err("precondition");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    for detail in [
        "Guard",
        "requires X > 0;",
        "X: 0",
        "Text: 50% 😀",
        "Values: [1, 2]",
        "Value: 3",
    ] {
        assert!(
            error.message.contains(detail),
            "{detail}: {}",
            error.message
        );
    }
    assert!(!error.message.contains("body was entered"));
    assert!(!error.message.contains("requires 10 div X"));
}

#[test]
fn every_entry_checks_direct_indirect_method_static_and_nested_calls() {
    for call in [
        "Guard(0)",
        "F(0)",
        "B.Guard(0)",
        "Bound(0)",
        "Box.StaticGuard(0)",
        "Nested(0)",
    ] {
        let source = format!("program T; type Box = record Value: integer;
            procedure Guard(Self: Box; X: integer); requires Self.Value > 0; requires X > 0; begin null; end procedure;
            static procedure StaticGuard(X: integer); requires X > 0; begin null; end procedure;
        end record;
        procedure Guard(X: integer); requires X > 0; begin null; end procedure;
        procedure Outer(Limit: integer);
            procedure Nested(X: integer); requires X >= Limit; begin null; end procedure;
            begin const F: procedure(X: integer) := Guard; const B := Box(Value := 1);
                const Bound: procedure(X: integer) := B.Guard; {call}; end procedure;
        begin Outer(1); end.");
        let error = run_program(&source).expect_err("each call must check its entry");
        assert!(
            error.message.contains("requires") && error.message.contains("X: 0"),
            "{call}: {error:?}"
        );
    }
}

#[test]
fn valid_clauses_preserve_var_parameters_generic_values_and_immutable_captures() {
    assert_succeeds(
        "program T;
        procedure Set(var X: integer); requires X = 1; begin X := 2; end procedure;
        function Identity<T: Comparable>(X: T): T; requires X = X; begin return X; end function;
        function Make(Limit: integer): procedure(X: integer);
            procedure Guard(X: integer); requires X >= Limit; begin null; end procedure;
            begin return Guard; end function;
        begin var X := 1; Set(var X); const F: procedure(X: integer) := Make(2); F(2);
            if X <> 2 or Identity('a') <> 'a' then panic('entry changes'); end if; end.",
    );
    let error = run_program(
        "program T; procedure P(X: integer); requires X < 3; begin null; end procedure;
        begin for I: integer := 1 to 3 do P(I); end for; end.",
    )
    .expect_err("later entry");
    assert!(error.message.contains("X: 3"), "{error:?}");
}

#[test]
fn restricted_if_case_and_native_queries_execute_at_entry() {
    assert_succeeds("program T; type Box = record Value: integer := 1; end record;
        procedure P(X: integer; Text: string; D: dict of string to integer);
        requires X > 0 and 10 div X > 0;
        requires (X in [X]) and ('a' in Text) and ('a' in D);
        requires Box().Value = 1;
        requires if Text.IsEmpty() then false else Text.Length() > 0 end if;
        requires case Some(X) of when Some(const N) if N > 0: true; when Some(_): false; when None: false; end case;
        begin null; end procedure; begin P(1, 'a', ['a': 1]); end.");
}

#[test]
fn clause_captures_precede_body_shadowing_and_defaults_keep_the_definition_scope() {
    assert_succeeds("program T; const Default: integer := 1;
        procedure Check(Default: integer); requires Box().Value = 1; begin null; end procedure;
        type Box = record Value: integer := Default; end record;
        function Make(Minimum: integer): procedure(Value: integer);
            procedure Guard(Value: integer); requires Value >= Minimum;
            begin const Minimum := 100; if Minimum <> 100 then panic('local'); end if; end procedure;
            begin return Guard; end function;
        begin Check(0); const Guard: procedure(Value: integer) := Make(2); Guard(2); end.");
    let error = run_program(
        "program T; function Guard<T: Comparable>(Value: T): T;
        requires false; begin return Value; end function;
        begin discard Guard('generic value'); end.",
    )
    .expect_err("generic entry");
    assert!(error.message.contains("Value: generic value"), "{error:?}");
}
