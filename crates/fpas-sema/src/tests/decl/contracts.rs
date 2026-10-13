use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_INVALID_CONTRACT_EXPRESSION, SEMA_TYPE_MISMATCH,
    SEMA_UNKNOWN_NAME,
};

#[test]
fn clauses_accept_immutable_values_parameters_construction_and_native_queries() {
    check_ok("program T; type Small = 0..10; type Tag = distinct integer;
        type Box = record Value: integer := 1; end record;
        function Read(): integer; begin return 1; end function;
        const Limit: integer := Read();
        procedure Check(var X: integer; Text: string; Values: array of integer; D: dict of string to integer; Callback: function(): integer);
        requires X >= Limit and X in Small and integer(Tag(X)) = X;
        requires Text.Length() > 0 and not Text.IsEmpty() and Text.Contains('a');
        requires Values.Length() > 0 and not Values.IsEmpty() and Values.Contains(X);
        requires D.Length() > 0 and not D.IsEmpty() and D.ContainsKey('a') and D['a'] = X;
        requires Box().Value = 1 and [X].Contains(X);
        requires [Callback].Length() = 1;
        requires Some(X) is Some(_);
        requires (Ok(Some(X)) is Ok(Some(_))) and (None is None);
        requires if X > 0 then true else false end if;
        requires case Some(X) of when Some(const N) if N > 0: true; when Some(_): false; when None: false; end case;
        begin null; end procedure; begin end.");
}

#[test]
fn postcondition_names_have_the_complete_result_type_and_only_clause_scope() {
    check_ok("program T;
        function Identity<T: Comparable>(X: T): T; ensures (Value): Value = X; begin return X; end function;
        function Read(X: integer): Result of (integer, string);
        ensures (Value): case Value of when Ok(const N): N = X; when Error(const Message): Message.Length() > 0; end case;
        ensures (Other): case Other of when Ok(_): true; when Error(_): true; end case;
        begin return Ok(X); end function;
        procedure Set(var X: integer); ensures X = 2; begin X := 2; end procedure; begin end.");
    for (source, code) in [
        (
            "function F(): integer; ensures (Value): Value = 'text'; begin return 1; end function;",
            SEMA_TYPE_MISMATCH,
        ),
        (
            "function F(): integer; ensures (Value): Value > 0; ensures Value > 0; begin return 1; end function;",
            SEMA_UNKNOWN_NAME,
        ),
        (
            "function F(): integer; ensures (Value): Value > 0; begin return Value; end function;",
            SEMA_UNKNOWN_NAME,
        ),
        (
            "function F(): integer; ensures Local > 0; begin const Local := 1; return Local; end function;",
            SEMA_UNKNOWN_NAME,
        ),
        (
            "function F(): Option of integer; ensures (Value): Value > 0; begin return Some(1); end function;",
            SEMA_TYPE_MISMATCH,
        ),
        (
            "function F(): integer; ensures 1; begin return 1; end function;",
            SEMA_TYPE_MISMATCH,
        ),
    ] {
        assert!(
            check_errors(&format!("program T; {source} begin end."))
                .iter()
                .any(|error| error.code == code),
            "{source}"
        );
    }
    let errors = check_errors(
        "program T; procedure P(); ensures (Value): true; begin null; end procedure; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION
                && error.message.contains("procedure"))
    );
    let errors = check_errors(
        "program T; uses Std.Console as C; function F(): integer;
        ensures (C): true; begin return 1; end function; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_DUPLICATE_DECLARATION
                && error.message.contains("import alias"))
    );
    for condition in [
        "Read() > 0",
        "Global > 0",
        "[Read].Length() > 0",
        "try Ok(1) > 0",
        "Box().Value > 0",
    ] {
        let source = format!(
            "program T; var Global: integer := 1;
            function Read(): integer; begin return 1; end function;
            type Box = record Value: integer := Read(); end record;
            function F(): integer; ensures {condition}; begin return 1; end function; begin end."
        );
        assert!(
            check_errors(&source)
                .iter()
                .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION),
            "{condition}"
        );
    }
    let errors = check_errors(
        "program T; procedure Outer(var X: integer);
        procedure Inner(); ensures X > 0; begin null; end procedure;
        begin Inner(); end procedure; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION)
    );
}

#[test]
fn clauses_reject_effects_and_mutable_state_recursively_by_resolved_identity() {
    let prefix = "program T;
        var Global: integer := 1; var Text: string := 'a';
        type Box = record Value: integer := 1;
            function Length(Self: Box): integer; begin return Self.Value; end function;
        end record;
        function Length(X: integer): integer; begin return X; end function;
        function Read(): integer; begin return 1; end function;
        procedure Check(X: integer; Callback: function(): integer; B: Box);";
    for clause in [
        "Global > 0",
        "Text.Length() > 0",
        "Read() > 0",
        "Length(X) > 0",
        "Callback() > 0",
        "Box().Length() > 0",
        "'a'.Slice(0, 1) = 'a'",
        "[Read()].Length() > 0",
        "[Read].Length() > 0",
        "[B.Length].Length() > 0",
        "[Box().Length].Length() > 0",
        "if true then Read() > 0 else false end if",
        "Some(Read()) is Some(_)",
        "Some(Global) is Some(_)",
        "Some(Callback()) is Some(_)",
        "case X of when const N if Read() > 0: true; else false; end case",
        "try Some(true)",
        "old(X) > 0",
        "(go Read()) = (go Read())",
        "(function(): boolean begin return true; end function) = Callback",
    ] {
        let source = format!("{prefix} requires {clause}; begin null; end procedure; begin end.");
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION),
            "{clause}: {errors:?}"
        );
    }
}

#[test]
fn clauses_reject_mutable_captures_and_body_locals_but_require_boolean_values() {
    let errors = check_errors(
        "program T; procedure Outer(var X: integer);
        procedure Inner(); requires X > 0; begin null; end procedure;
        begin Inner(); end procedure; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION)
    );
    let errors =
        check_errors("program T; procedure P(); requires 1; begin null; end procedure; begin end.");
    assert!(errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH));
    let errors = check_errors(
        "program T; procedure P(); requires Local > 0; begin const Local := 1; end procedure; begin end.",
    );
    assert!(errors.iter().any(|error| error.message.contains("Local")));
}

#[test]
fn omitted_defaults_keep_their_declaration_scope_and_contract_restrictions() {
    for default in ["Read()", "Global"] {
        let source = format!(
            "program T; var Global: integer := 1;
            function Read(): integer; begin return 1; end function;
            type Box = record Value: integer := {default}; end record;
            procedure P(); requires Box().Value > 0; begin null; end procedure; begin end."
        );
        assert!(
            check_errors(&source)
                .iter()
                .any(|error| error.code == SEMA_INVALID_CONTRACT_EXPRESSION),
            "{default}"
        );
        check_ok(&source.replace("Box().Value", "Box(Value := 1).Value"));
    }
    check_ok(
        "program T; const Default: integer := 1;
        procedure P(Default: integer); requires Box().Value > 0; begin null; end procedure;
        type Box = record Value: integer := Default; end record; begin P(0); end.",
    );
}
