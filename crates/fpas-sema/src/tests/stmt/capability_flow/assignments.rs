use super::*;

const TYPES: &str = "type Pair = record First: function(): integer; Second: function(): integer; Number: integer; end record;";

#[test]
fn field_and_index_writes_preserve_other_local_paths() {
    for body in [
        "var Box := Pair(First := Free, Second := Free, Number := 1); Box.First := Local; Box.Second := Free; Box.Number := 2; discard go Box.First();",
        "var Items: array of function(): integer := [Free, Free]; Items[0] := Local; Items[1] := Free; discard go Items[0]();",
        "var Items: dict of string to function(): integer := ['left': Free, 'right': Free]; Items['left'] := Local; Items['right'] := Free; discard go Items['left']();",
    ] {
        rejects_local(TYPES, body);
    }
}

#[test]
fn whole_binding_overwrites_restore_transferability() {
    for body in [
        "var Box := Pair(First := Local, Second := Free, Number := 1); Box := Pair(First := Free, Second := Free, Number := 2); discard go Box.First();",
        "var Items: array of function(): integer := [Local]; Items := [Free]; discard go Items[0]();",
        "Chosen := Local; Chosen := Free; discard go Chosen();",
    ] {
        check_ok(&program(TYPES, body));
    }
}

#[test]
fn scalar_projections_of_local_records_remain_transferable() {
    check_ok(&program(
        TYPES,
        "const Box := Pair(First := Local, Second := Free, Number := 7); const Number := Box.Number; const Read: function(): integer := function(): integer begin return Number; end function; discard go Read();",
    ));
}
