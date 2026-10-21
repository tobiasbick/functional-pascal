use super::*;

#[test]
fn for_in_inherits_callable_elements_but_keeps_scalar_keys_free() {
    rejects_local(
        "",
        "for Item: function(): integer in [Local] do discard go Item(); end for;",
    );
    check_ok(&program(
        "",
        "for Item: function(): integer in [Free] do discard go Item(); end for;",
    ));
    check_ok(&program(
        "",
        "const Items: dict of integer to function(): integer := [7: Local]; for Key: integer in Items do const Read: function(): integer := function(): integer begin return Key; end function; discard go Read(); end for;",
    ));
}

#[test]
fn enum_and_nested_wrapper_patterns_preserve_local_callable_contents() {
    let types =
        "type WorkChoice = enum Job(Work: function(): integer); Number(Value: integer); end enum;";
    rejects_local(
        types,
        "const Wrapped: option of WorkChoice := Some(WorkChoice.Job(Local)); case Wrapped of when Some(WorkChoice.Job(const Work)): discard go Work(); when Some(WorkChoice.Number(_)), None: null; end case;",
    );
    check_ok(&program(
        types,
        "const Wrapped: WorkChoice := WorkChoice.Job(Local); case Wrapped of when WorkChoice.Number(const Number): const Read: function(): integer := function(): integer begin return Number; end function; discard go Read(); when WorkChoice.Job(_): null; end case;",
    ));
}
