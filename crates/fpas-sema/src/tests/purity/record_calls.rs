//! Record accessors obey the ordinary-call purity boundary, including defaults.

use crate::tests::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

const DECLARATIONS: &str = "
    var State: integer := 0;
    type Handler = procedure();
    type Box = record Count: integer := 0; end record;
    function BoxReadCount(Receiver: Box): integer;
    begin State := State + 1; return State; end function;
    procedure BoxWriteCount(Receiver: Box; Value: integer); begin State := Value; end procedure;
    function BoxReadCallback(Receiver: Box): pure function(): integer;
    begin State := State + 1;
      return pure function(): integer begin return 42; end function;
    end function;
    function BoxReadAction(Receiver: Box): Option of (Handler);
    begin State := State + 1; return Option.None; end function;
    procedure BoxWriteAction(Receiver: Box; Value: Option of (Handler));
    begin State := State + 1; end procedure;
    type Wrapper = record Value: Box; end record;
    function WrapperReadChild(Receiver: Wrapper): Box; begin return Receiver.Value; end function;
    const Instance: Box := Box();
    const Items: array of (Box) := [Instance];
    const Outer: Wrapper := Wrapper(Value := Instance);
";

#[test]
fn pure_functions_and_defaults_reject_accessors_with_every_record_argument_form() {
    for expression in [
        "BoxReadCount(Instance)",
        "BoxReadCount((Instance))",
        "BoxReadCount(Items[0])",
        "BoxReadCount(WrapperReadChild(Outer))",
        "BoxReadCount(WrapperReadChild((Outer)))",
        "BoxReadCallback(Instance)()",
        "(BoxReadCallback(Instance))()",
        "if true then BoxReadCount(Instance) else 0 end if",
        "case 1 of when 1: BoxReadCount(Instance); else 0; end case",
    ] {
        for declaration in [
            format!("type Settings = record Value: integer := {expression}; end record;"),
            format!("pure function Evaluate(): integer; begin return {expression}; end function;"),
        ] {
            let errors = check_errors(&format!(
                "program Main; {DECLARATIONS} {declaration} begin null; end program;"
            ));
            assert!(
                errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                    && error.message.contains("Pure evaluation cannot call")),
                "{declaration}: {errors:#?}"
            );
        }
    }
}

#[test]
fn ordinary_evaluation_and_deferred_ordinary_bodies_allow_accessors() {
    check_ok(&format!(
        "program Main; {DECLARATIONS}
        type Settings = record
          Action: function(): integer := function(): integer begin return BoxReadCount(Instance); end function;
        end record;
        function Evaluate(): integer; begin return BoxReadCount(WrapperReadChild(Outer)); end function;
        begin
          discard Evaluate(); discard BoxReadCount(Items[0]); discard BoxReadCallback(Instance)();
          const Value := Settings(); discard Value.Action();
          BoxWriteCount(Instance, 7); BoxWriteAction(Instance, Option.None);
          case BoxReadAction(Instance) of when Option.Some(const Action): Action(); when Option.None: null; end case;
        end program;"
    ));
}

#[test]
fn pure_evaluation_rejects_optional_handler_accessors_and_setters() {
    for declaration in [
        "type Bad = record Action: Option of (Handler) := BoxReadAction(Instance); end record;",
        "pure function Evaluate(): integer; begin discard BoxReadAction(Instance); return 0; end function;",
        "pure function Evaluate(): integer; begin const Local := Box(); BoxWriteCount(Local, 7); return 0; end function;",
        "pure function Evaluate(): integer; begin const Local := Box(); BoxWriteAction(Local, Option.None); return 0; end function;",
    ] {
        let errors = check_errors(&format!(
            "program Main; {DECLARATIONS} {declaration} begin null; end program;"
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TYPE_MISMATCH
                && error.message.contains("Pure evaluation cannot call")),
            "{declaration}: {errors:#?}"
        );
    }
}
