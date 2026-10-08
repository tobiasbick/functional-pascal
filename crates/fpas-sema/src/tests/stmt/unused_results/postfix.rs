use super::*;

const RECORD: &str = "type Box = record Value: integer;
  function Next(Self: Box): Box; begin return Self; end function;
  function Get(Self: Box): integer; begin return Self.Value; end function;
  procedure Print(Self: Box); begin end procedure;
  end record;
  function Make(): Box; begin return Box( Value := 1 ); end function;";

#[test]
fn member_and_postfix_function_calls_require_consumption() {
    for statement in ["Make().Get();", "Make().Next().Get();", "Value.Get();"] {
        unused(&format!(
            "program T; {RECORD}
          begin const Value: Box := Make(); {statement} end."
        ));
    }
}

#[test]
fn final_procedures_and_consumed_chains_remain_valid() {
    check_ok(&format!(
        "program T; {RECORD}
      begin Make().Print(); Make().Next().Print(); discard Make().Next().Get();
      const N: integer := Make().Get(); end."
    ));
}

#[test]
fn receiver_calls_and_callable_record_members_require_consumption() {
    unused("program T;
      function Plus(Value: integer; Other: integer): integer; begin return Value + Other; end function;
      begin [1].Length(); end.");
    unused(
        "program T;
      type Box = record Get: function(): integer; end record;
      begin const Value: Box := Box( Get := function(): integer begin return 1; end function );
      Value.Get(); end.",
    );
}

#[test]
fn postfix_callable_hints_agree_with_explicit_discard_rules() {
    let error = unused(
        "program T;
      type Box = record Value: integer;
        function Make(Self: Box): function(): integer;
        begin return function(): integer begin return Self.Value; end function; end function;
      end record;
      function Build(): Box; begin return Box( Value := 1 ); end function;
      begin Build().Make(); end.",
    );
    assert!(error.help.unwrap().contains("`discard Call();`"));
    check_ok(
        "program T;
      type Box = record Value: integer;
        function Make(Self: Box): function(): integer;
        begin return function(): integer begin return Self.Value; end function; end function;
      end record;
      function Build(): Box; begin return Box( Value := 1 ); end function;
      begin discard Build().Make(); end.",
    );
}

#[test]
fn postfix_task_results_do_not_recommend_invalid_discard() {
    let error = unused(
        "program T;
      function Work(): integer; begin return 1; end function;
      type Box = record
        function Start(Self: Box): task of integer;
        begin return go Work(); end function;
      end record;
      function Make(): Box; begin return Box( ); end function;
      begin Make().Start(); end.",
    );
    assert!(!error.help.unwrap().contains("`discard"));
}
