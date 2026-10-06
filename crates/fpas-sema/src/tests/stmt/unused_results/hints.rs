use super::*;

#[test]
fn result_hint_names_case_try_and_safe_discard() {
    let error = unused(
        "program T;
      function Value(): result of integer, string; begin return Error('failed'); end function;
      begin Value(); end.",
    );
    let help = error.help.unwrap();
    assert!(
        help.contains("`case`") && help.contains("`try`") && help.contains("`discard Call();`")
    );
}

#[test]
fn task_results_and_aggregates_never_suggest_discard_as_a_fix() {
    for (ty, value) in [
        ("task of integer", "go Worker()"),
        ("Option of task of integer", "None"),
        ("array of task of integer", "[]"),
        ("result of integer, task of integer", "Ok(1)"),
        ("channel of task of integer", "CreateChannel(1)"),
    ] {
        let error = unused(&format!(
            "program T; uses Std.Tasks;
          function Worker(): integer; begin return 1; end function;
          function Produce(): {ty}; begin return {value}; end function;
          begin Produce(); end."
        ));
        let help = error.help.unwrap();
        assert!(!help.contains("`discard"), "{help}");
        assert!(!help.contains("go Worker"), "{help}");
    }
}

#[test]
fn callable_results_use_known_capture_proofs() {
    let safe = unused(
        "program T;
      function Make(): function(): integer;
      begin return function(): integer begin return 1; end function; end function;
      begin Make(); end.",
    );
    assert!(safe.help.unwrap().contains("`discard Call();`"));
    let unknown = unused(
        "program T;
      procedure Ignore(Factory: function(): function(): integer);
      begin Factory(); end procedure; begin end.",
    );
    assert!(!unknown.help.unwrap().contains("`discard"));
    let captured = unused(
        "program T; uses Std.Tasks;
      function Make(Job: task of integer): function(): integer;
      begin return function(): integer begin return Wait(Job); end function; end function;
      function Work(): integer; begin return 1; end function;
      begin var Job: task of integer := go Work(); Make(Job); end.",
    );
    assert!(!captured.help.unwrap().contains("`discard"));
}

#[test]
fn generic_body_constraints_determine_discard_hint() {
    for (constraint, safe) in [("", false), (": Printable", false), (": Numeric", true)] {
        let error = unused(&format!(
            "program T;
          function Identity<T{constraint}>(Value: T): T; begin return Value; end function;
          procedure Ignore<T{constraint}>(Value: T); begin Identity(Value); end procedure;
          begin end."
        ));
        assert_eq!(error.help.unwrap().contains("`discard Call();`"), safe);
    }
}
