//! Return paths, clause scope, and diagnostics observed through compiled execution.
use super::{assert_succeeds, run_program};

#[test]
fn grouped_boolean_names_are_ordinary_function_and_procedure_postconditions() {
    for (declaration, call) in [
        (
            "function Verify(Flag: boolean): boolean; ensures (Flag); begin return true; end function;",
            "discard Verify",
        ),
        (
            "procedure Verify(Flag: boolean); ensures (Flag); begin null; end procedure;",
            "Verify",
        ),
    ] {
        assert_succeeds(&format!(
            "program T; {declaration} begin {call}(true); end."
        ));
        let error = run_program(&format!(
            "program T; {declaration} begin {call}(false); end."
        ))
        .expect_err("grouped parameter must remain the condition");
        assert!(
            error.message.contains("ensures (Flag);") && error.message.contains("Flag: false"),
            "{error:?}"
        );
    }
}

#[test]
fn every_early_and_final_return_checks_clauses_in_order_with_complete_values() {
    for input in [-1, 0, 1] {
        let source = format!(
            "program T;
            function Guard(X: integer): integer;
            ensures (Value): Value > 1; ensures 10 div X > 0;
            begin if X < 0 then return -1; end if;
                while X = 0 do return 0; end while; return 1; end function;
            begin const Value: integer := Guard({input}); end."
        );
        let error = run_program(&source).expect_err("postcondition on every return");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        for detail in [
            "T.Guard",
            "ensures (Value): Value > 1;",
            &format!("X: {input}"),
            &format!("Return: {input}"),
        ] {
            assert!(error.message.contains(detail), "{detail}: {error:?}");
        }
    }
    let error = run_program(
        "program T; type Box = record Value: integer; end record;
        function F(): Box; ensures false; begin return Box(Value := 3); end function;
        begin const B: Box := F(); end.",
    )
    .expect_err("aggregate return");
    assert!(error.message.contains("Return: Box{Value: 3}"), "{error:?}");
}

#[test]
fn try_error_and_none_exits_check_the_complete_declared_return_value() {
    for (result, operand, condition, returned) in [
        (
            "Result of (integer, string)",
            "Error('failure')",
            "case Value of when Ok(_): true; when Error(_): false; end case",
            "Error(failure)",
        ),
        (
            "Option of integer",
            "None",
            "case Value of when Some(_): true; when None: false; end case",
            "None",
        ),
    ] {
        let source = format!(
            "program T; function F(var X: integer): {result};
            ensures (Value): {condition};
            begin X := 2; const Input: {result} := {operand};
                const N: integer := try Input; panic('after try'); end function;
            begin var X := 1; discard F(var X); end."
        );
        let error = run_program(&source).expect_err("propagated return must check postconditions");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(
            error.message.contains("ensures (Value)")
                && error.message.contains(returned)
                && error.message.contains("X: 2"),
            "{error:?}"
        );
        assert!(!error.message.contains("after try"));
        assert_succeeds(&source.replace(condition, "true"));
    }
    assert_succeeds("program T;
        function F(): Option of integer; ensures (Value): case Value of when Some(const N): N = 3; when None: false; end case;
        begin const Input: Option of integer := Some(3); const N: integer := try Input; return Some(N); end function;
        begin discard F(); end.");
}

#[test]
fn implicit_and_explicit_procedure_returns_read_current_var_parameters() {
    for ending in ["return;", "null;"] {
        assert_succeeds(&format!(
            "program T; procedure Set(var X: integer); requires X = 1; ensures X = 2;
            begin X := 2; {ending} end procedure; begin var X := 1; Set(var X);
            if X <> 2 then panic('lost write'); end if; end."
        ));
        let error = run_program(&format!(
            "program T; procedure Set(var X: integer); ensures X = 1;
            begin X := 2; {ending} end procedure; begin var X := 1; Set(var X); end."
        ))
        .expect_err("current parameter");
        assert!(
            error.message.contains("X: 2") && error.message.contains("ensures X = 1;"),
            "{error:?}"
        );
    }
}

#[test]
fn worker_local_var_writes_precede_grouped_postcondition_failure() {
    assert_succeeds(
        "program T; uses Std.Tasks as Tasks;\n        procedure Set(var X: integer; Observed: channel of integer);\n        ensures X = 1; begin X := 2; discard Tasks.Send(Observed, X); end procedure;\n        procedure Verify(); begin\n            const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();\n            const Observed: channel of integer := Tasks.CreateChannel(1);\n            const Child: task := Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken)\n                begin var Local := 1; Set(var Local, Observed); end procedure);\n            const Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(Group);\n            if Tasks.Receive(Observed).Unwrap() <> 2 or Failures.Length() <> 1 then panic('lost write or failure'); end if;\n        end procedure; begin Verify(); end.",
    );
}

#[test]
fn direct_indirect_bound_static_nested_and_tail_returns_cannot_bypass_checks() {
    for call in [
        "Guard(0)",
        "F(0)",
        "B.Read()",
        "Bound()",
        "Box.ReadStatic()",
        "Tail(0)",
        "Nested(0)",
    ] {
        let source = format!("program T; type Box = record Value: integer;
            function Read(Self: Box): integer; ensures (Value): Value > 0; begin return Self.Value; end function;
            static function ReadStatic(): integer; ensures (Value): Value > 0; begin return 0; end function;
        end record;
        function Guard(X: integer): integer; ensures (Value): Value > 0; begin return X; end function;
        function Plain(X: integer): integer; begin return X; end function;
        function Tail(X: integer): integer; ensures (Value): Value > 0; begin return Plain(X); end function;
        procedure Outer(Limit: integer);
            function Nested(X: integer): integer; ensures (Value): Value >= Limit; begin return X; end function;
            begin const F: function(X: integer): integer := Guard; const B := Box(Value := 0);
            const Bound: function(): integer := B.Read; const Value: integer := {call}; end procedure;
        begin Outer(1); end.");
        let error = run_program(&source).expect_err("callee postcondition");
        assert!(
            error.message.contains("ensures (Value)") && error.message.contains("Return: 0"),
            "{call}: {error:?}"
        );
    }
}

#[test]
fn postcondition_scope_survives_body_shadowing_and_result_names_are_independent() {
    assert_succeeds("program T; const Limit: integer := 1;
        function Make(Minimum: integer): function(X: integer): integer;
            function Guard(X: integer): integer;
            ensures (Value): Value >= Minimum; ensures (Other): Other >= Limit;
            begin const Minimum := 100; const Limit := 100; return X; end function;
            begin return Guard; end function;
        function Identity<T: Comparable>(X: T): T; ensures (Value): Value = X; begin return X; end function;
        function Shadow(Value: integer): integer; ensures (Value): Value = 2; begin return 2; end function;
        begin const F: function(X: integer): integer := Make(2);
            if F(2) <> 2 or Identity('a') <> 'a' or Shadow(1) <> 2 then panic('scope'); end if; end.");
    let error = run_program(
        "program T; function Shadow(Value: integer): integer;
        ensures (Value): Value > 2; begin return 2; end function;
        begin const N: integer := Shadow(1); end.",
    )
    .expect_err("shadowed parameter details");
    assert!(
        error.message.contains("Parameters: {Value: 1}") && error.message.contains("Return: 2"),
        "{error:?}"
    );
}

#[test]
fn postconditions_do_not_supply_missing_returns_even_when_always_false() {
    for condition in ["true", "false"] {
        for body in ["null;", "if X > 0 then return X; end if;"] {
            let program = super::super::parse_ok(&format!("program T;
                function F(X: integer): integer; ensures (Value): {condition}; begin {body} end function;
                begin const N: integer := F(0); end."));
            assert!(crate::compile(&program).is_err(), "{condition}: {body}");
        }
    }
}
