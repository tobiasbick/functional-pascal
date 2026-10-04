use super::rejects;
use crate::tests::check_ok;

#[test]
fn pure_bodies_reject_effects_and_outer_mutable_storage() {
    for body in [
        "return Ordinary();",
        "return State;",
        "State := 2; return 1;",
        "discard go Clean(); return 1;",
    ] {
        rejects(&format!(
            "program Main; var State: integer := 0;
            function Ordinary(): integer; begin return 1; end function;
            pure function Clean(): integer; begin return 1; end function;
            pure function Bad(): integer; begin {body} end function;
            begin null; end program;"
        ));
    }
}

#[test]
fn pure_closures_reject_mutable_and_transitively_mutable_captures() {
    for closure in [
        "pure function(): integer begin return State; end function",
        "pure function(): integer begin const Inner := function(): integer begin return State; end function; return 1; end function",
    ] {
        rejects(&format!(
            "program Main; function Factory(): function(): integer;
            begin var State := 0; return {closure}; end function; begin null; end program;"
        ));
    }
    check_ok(
        "program Main; pure function Value(): integer;
        function Ordinary(): integer; begin return 2; end function;
        begin var Local := 1; Local := Local + 1; return Local; end function;
        begin discard Value(); end program;",
    );
}
