//! Initialization checks are asserted at the language-analysis boundary.

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_UNINITIALIZED_DEPENDENCY;

#[test]
fn forward_calls_and_mutual_recursion_work_in_every_named_scope() {
    check_ok(
        "program T;
        const Initial: integer := Twice(3);
        function Twice<T: Numeric>(Value: T): T; begin return Value + Value; end function;
        function Even(Value: integer): boolean;
        begin return if Value = 0 then true else Odd(Value - 1) end if; end function;
        function Odd(Value: integer): boolean;
        begin return if Value = 0 then false else Even(Value - 1) end if; end function;
        function Outer(): integer;
            function First(Value: integer): integer; begin return Second(Value); end function;
            function Second(Value: integer): integer; begin return Value; end function;
        begin return First(Initial); end function;
        begin discard Outer(); discard Even(8); end.",
    );
}

#[test]
fn initialization_rejects_direct_transitive_default_and_capture_access() {
    for source in [
        "const First: integer := Helper(); function Helper(): integer; begin return Later; end function; const Later: integer := 7;",
        "const First: integer := Helper(); function Helper(): integer; begin return ReadLater(); end function; function ReadLater(): integer; begin return Later; end function; const Later: integer := 7;",
        "const First: Box := Box(); type Box = record Value: integer := ReadLater(); end record; function ReadLater(): integer; begin return Later; end function; const Later: integer := 7;",
        "function Outer(): integer; function ReadLater(): integer; begin return Later; end function; begin const First := ReadLater(); const Later := 7; return First; end function;",
        "function Outer(): function(): integer; function ReadLater(): integer; begin return Later; end function; begin const First: function(): integer := ReadLater; const Later := 7; return First; end function;",
        "function Outer(): integer; function SetLater(): integer; begin Later := 9; return 1; end function; begin const First := SetLater(); var Later := 7; return First; end function;",
    ] {
        let errors = check_errors(&format!("program T; {source} begin end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
            "{source}: {errors:#?}"
        );
        assert!(
            errors.iter().any(|error| error.message.contains("Later")),
            "{errors:#?}"
        );
    }
}

#[test]
fn known_callbacks_cannot_hide_premature_dependencies() {
    let source = "program T;
        const First: integer := Apply(ReadLater);
        const Later: integer := 7;
        function Apply(F: function(): integer): integer; begin return F(); end function;
        function ReadLater(): integer; begin return Later; end function;
        begin end.";
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY
                && error.message.contains("Later")),
        "{errors:#?}"
    );
    check_ok(
        "program T; const Later: integer := 7;
        const First: integer := Apply(ReadLater);
        function Apply(F: function(): integer): integer; begin return F(); end function;
        function ReadLater(): integer; begin return Later; end function;
        begin discard First; end.",
    );
}

#[test]
fn supplied_fields_skip_unsafe_defaults_and_safe_calls_follow_initialization() {
    check_ok(
        "program T;
        const First: Box := Box(Value := 3);
        type Box = record Value: integer := ReadLater(); end record;
        function ReadLater(): integer; begin return Later; end function;
        const Later: integer := 7;
        const Second: Box := Box();
        begin discard First; discard Second; end.",
    );
}

#[test]
fn mutable_callable_storage_cannot_keep_stale_initialization_proofs() {
    let errors = check_errors(
        "program T;
        var Stored: function(): integer := Safe;
        const Changed: integer := Replace();
        const First: integer := Stored();
        const Later: integer := 7;
        function Safe(): integer; begin return 1; end function;
        function Replace(): integer; begin Stored := ReadLater; return 0; end function;
        function ReadLater(): integer; begin return Later; end function;
        begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
        "{errors:#?}"
    );
}

#[test]
fn returned_captured_callbacks_remain_conservative_during_startup() {
    let errors = check_errors(
        "program T;
        const Stored: function(): integer := Wrap(ReadLater);
        const First: integer := Stored();
        const Later: integer := 7;
        function Wrap(F: function(): integer): function(): integer;
            function Run(): integer; begin return F(); end function;
        begin return Run; end function;
        function ReadLater(): integer; begin return Later; end function;
        begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
        "{errors:#?}"
    );
}

#[test]
fn recursive_callable_bindings_terminate_analysis_and_remain_conservative() {
    let declarations = "var Stored: function(Input: integer): integer := CountDown;
        function CountDown(Input: integer): integer;
        begin
            if Input = 0 then return 1; else return Stored(Input - 1); end if;
        end function;";
    let errors = check_errors(&format!(
        "program T; {declarations} const First: integer := Stored(3); begin end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
        "{errors:#?}"
    );
    check_ok(&format!(
        "program T; {declarations} begin discard Stored(3); end."
    ));
}

#[test]
fn postfix_var_arguments_invalidate_callable_initialization_proofs() {
    let errors = check_errors(
        "program T;
        var Stored: function(): integer := Safe;
        const Changed: integer := Holder().Replace(var Stored);
        const First: integer := Stored();
        const Later: integer := 7;
        type Holder = record
            function Replace(Self: Holder; var Target: function(): integer): integer;
            begin Target := ReadLater; return 0; end function;
        end record;
        function Safe(): integer; begin return 1; end function;
        function ReadLater(): integer; begin return Later; end function;
        begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_UNINITIALIZED_DEPENDENCY),
        "{errors:#?}"
    );
}
