//! Read-only iteration is enforced at the complete language-analysis boundary.

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_ITERATION_SOURCE_WRITE;

fn rejects_source_write(source: &str) {
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_ITERATION_SOURCE_WRITE),
        "{source}\n{errors:#?}"
    );
}

#[test]
fn for_in_rejects_direct_array_and_dictionary_writes() {
    for (declaration, item_type, statement) in [
        ("var Items := [1, 2];", "integer", "Items.PushInPlace(3);"),
        (
            "var Items := [1, 2];",
            "integer",
            "discard Items.PopInPlace();",
        ),
        ("var Items := [1, 2];", "integer", "Items[0] := 9;"),
        ("var Items := [1, 2];", "integer", "Items := [9];"),
        ("var Items := [1, 2];", "integer", "Items.SortInPlace();"),
        (
            "var Items: dict of string to integer := ['a': 1];",
            "string",
            "Items['a'] := 9;",
        ),
        (
            "var Items: dict of string to integer := ['a': 1];",
            "string",
            "Items['b'] := 9;",
        ),
        (
            "var Items: dict of string to integer := ['a': 1];",
            "string",
            "Items.RemoveInPlace('a');",
        ),
        (
            "var Items: dict of string to integer := ['a': 1];",
            "string",
            "Items := ['b': 9];",
        ),
    ] {
        rejects_source_write(&format!(
            "program T; begin {declaration} for Item: {item_type} in Items do {statement} end for; end."
        ));
    }
}

#[test]
fn for_in_checks_imported_writes_and_callback_substitution() {
    let (parsed, errors) = fpas_parser::parse_compilation_unit(
        "unit Storage;
         public var Items: array of integer := [1, 2];
         public var Values: dict of string to integer := ['a': 1];
         public procedure Change(); begin Replace(); end procedure;
         procedure Replace(); begin Items.PushInPlace(9); Values['a'] := 9; end procedure;
         public function Read(): integer; begin return Items.Length(); end function;
         public procedure Apply(Action: procedure()); begin Action(); end procedure;
         end unit;",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let fpas_parser::CompilationUnit::Unit(unit) = parsed else {
        panic!("expected unit");
    };
    let analyzed = crate::analyze_unit(&unit, &[]).expect("unit analysis");
    assert!(
        analyzed.metadata.errors.is_empty(),
        "{:#?}",
        analyzed.metadata.errors
    );
    let interface = analyzed.interface.expect("unit interface");
    let bytes = fpas_unit::interface::encode_interface(&interface).expect("interface encoding");
    let interface = fpas_unit::interface::decode_interface(&bytes).expect("interface decoding");
    for body in [
        "for Item: integer in S.Items do S.Change(); end for;",
        "for Key: string in S.Values do S.Change(); end for;",
        "var Items := [1, 2]; for Item: integer in Items do S.Apply(procedure() begin Items[0] := 9; end procedure); end for;",
        "for Item: integer in S.Items do S.Items[0] := 9; end for;",
        "for Key: string in S.Values do S.Values.RemoveInPlace(Key); end for;",
    ] {
        let (program, errors) =
            fpas_parser::parse(&format!("program T; uses Storage as S; begin {body} end."));
        assert!(errors.is_empty(), "{errors:#?}");
        let metadata =
            crate::analyze_program_with_interfaces(&program, std::slice::from_ref(&interface))
                .expect("program analysis");
        assert!(
            metadata
                .errors
                .iter()
                .any(|error| error.code == SEMA_ITERATION_SOURCE_WRITE),
            "{body}: {:#?}",
            metadata.errors
        );
    }
    let (program, errors) = fpas_parser::parse(
        "program T; uses Storage as S; begin for Item: integer in S.Items do discard S.Read(); end for; S.Change(); end.",
    );
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata =
        crate::analyze_program_with_interfaces(&program, &[interface]).expect("program analysis");
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
}

#[test]
fn for_in_rejects_transitive_helpers_closures_and_callbacks() {
    for declarations in [
        "procedure Change(); begin Items[0] := 9; end procedure;",
        "procedure Change(); begin Other(); end procedure; procedure Other(); begin Items.PushInPlace(9); end procedure;",
        "procedure Change(); begin Recurse(2); end procedure; procedure Recurse(N: integer); begin if N > 0 then Recurse(N - 1); end if; Items[0] := 9; end procedure;",
        "const Change: procedure() := procedure() begin Items[0] := 9; end procedure;",
        "procedure Apply(Action: procedure()); begin Action(); end procedure; procedure Change(); begin Apply(procedure() begin Items[0] := 9; end procedure); end procedure;",
        "procedure Change(); begin discard Items.Map(function(Value: integer): integer begin Items[0] := 9; return Value; end function); end procedure;",
    ] {
        rejects_source_write(&format!(
            "program T; var Items: array of integer := [1, 2]; {declarations} begin for Item: integer in Items do Change(); end for; end."
        ));
    }
}

#[test]
fn for_in_rejects_var_forwarding_containing_roots_and_outer_sources() {
    for source in [
        "program T; procedure Change(var Values: array of integer); begin Values.PushInPlace(9); end procedure; begin var Items := [1, 2]; for Item: integer in (Items) do Change(var Items); end for; end.",
        "program T; type Box = record Items: array of integer; end record; begin var State := Box(Items := [1, 2]); for Item: integer in State.Items do State := Box(Items := [9]); end for; end.",
        "program T; begin var Items := [[1, 2]]; for Item: integer in Items[0] do Items[0][0] := 9; end for; end.",
        "program T; begin var Items := [1, 2]; for Item: integer in Items do for Other: integer in [3] do Items.PushInPlace(9); end for; end for; end.",
        "program T; procedure Run(var Items: array of integer; Action: procedure()); begin for Item: integer in Items do Action(); end for; end procedure; begin end.",
        "program T; var Items: array of integer := [1, 2]; procedure Change(); begin Items[0] := 9; end procedure; procedure Run(var Source: array of integer); begin for Item: integer in Source do Change(); end for; end procedure; begin Run(var Items); end.",
        "program T; var Items: array of integer := [1, 2]; procedure Run(var Alias: array of integer); begin for Item: integer in Items do Alias[0] := 9; end for; end procedure; begin Run(var Items); end.",
        "program T; procedure Outer(); procedure Run(var Source: array of integer); begin for Item: integer in Source do Items[0] := 9; end for; end procedure; begin var Items := [1, 2]; Run(var Items); end procedure; begin Outer(); end.",
    ] {
        rejects_source_write(source);
    }
}

#[test]
fn for_in_rejects_deferred_captured_callable_writes() {
    rejects_source_write(
        "program T; procedure Outer();
         procedure InvokeChange(); begin Change(); end procedure;
         procedure Iterate(); begin for Item: integer in Items do InvokeChange(); end for; end procedure;
         begin var Items := [1, 2];
           var Change: procedure() := procedure() begin Items[0] := 9; end procedure;
           Iterate();
         end procedure; begin Outer(); end.",
    );
}

#[test]
fn for_in_allows_reads_copies_computed_sources_and_post_loop_writes() {
    for source in [
        "program T; var Items: array of integer := [1, 2]; function Read(): integer; begin return Items.Length(); end function; begin for Item: integer in Items do discard Read(); end for; Items.PushInPlace(3); end.",
        "program T; begin var Items := [1, 2]; var Copy := Items; for Item: integer in Items do Copy.PushInPlace(Item); end for; Items[0] := 3; end.",
        "program T; begin var Items := [1, 2]; for Item: integer in Items do begin var Items := [3]; Items.PushInPlace(Item); end; end for; end.",
        "program T; begin var Items: dict of string to integer := ['a': 1]; for Key: string in Items.Keys() do Items[Key] := 2; end for; end.",
        "program T; begin var Items := [1, 2]; for Item: integer in Items do const Later: procedure() := procedure() begin Items.PushInPlace(9); end procedure; discard Later; end for; end.",
        "program T; procedure Run(var Source: array of integer); begin var Copy := Source; for Item: integer in Source do Copy.PushInPlace(Item); end for; Source.PushInPlace(9); end procedure; begin var Items := [1, 2]; Run(var Items); end.",
    ] {
        check_ok(source);
    }
}
