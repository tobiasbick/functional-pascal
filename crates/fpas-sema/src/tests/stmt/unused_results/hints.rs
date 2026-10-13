use super::*;

#[test]
fn result_hint_names_case_try_and_discard() {
    let error = unused(
        "program T;
        function Value(): result of (integer, string); begin return Error('failed'); end function;
        begin Value(); end.",
    );
    let help = error.help.unwrap();
    assert!(
        help.contains("`case`") && help.contains("`try`") && help.contains("`discard Call();`")
    );
}

#[test]
fn every_value_result_offers_discard_independently_of_its_contents() {
    for ty in [
        "task of integer",
        "Option of task of integer",
        "array of task of integer",
        "result of (integer, task of integer)",
        "channel of task of integer",
        "function(): integer",
    ] {
        let error = unused(&format!(
            "program T;
            function Produce(Value: {ty}): {ty}; begin return Value; end function;
            procedure Ignore(Value: {ty}); begin Produce(Value); end procedure;
            begin end."
        ));
        assert!(error.help.unwrap().contains("`discard Call();`"), "{ty}");
    }
}

#[test]
fn generic_results_offer_discard_without_capture_or_constraint_proofs() {
    for constraint in ["", ": Printable", ": Numeric"] {
        let error = unused(&format!(
            "program T;
            function Identity<T{constraint}>(Value: T): T; begin return Value; end function;
            procedure Ignore<T{constraint}>(Value: T); begin Identity(Value); end procedure;
            begin end."
        ));
        assert!(error.help.unwrap().contains("`discard Call();`"));
    }
}
