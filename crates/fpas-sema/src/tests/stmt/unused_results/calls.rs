use super::*;

#[test]
fn ordinary_result_option_and_alias_results_require_consumption() {
    for (ty, value) in [
        ("integer", "42"),
        ("string", "'hello'"),
        ("result of (integer, string)", "Error('failure')"),
        ("Option of integer", "None"),
        ("array of integer", "[]"),
    ] {
        let error = unused(&format!(
            "program T; type Value = {ty};
             function Produce(): Value; begin return {value}; end function;
             begin Produce(); end."
        ));
        assert!(error.message.starts_with("Unused function result"));
        assert!(error.help.unwrap().contains("`discard Call();`"));
    }
}

#[test]
fn generic_and_callable_variable_calls_require_consumption() {
    unused(
        "program T;
      function Identity<T>(Value: T): T; begin return Value; end function;
      begin Identity(1); end.",
    );
    unused(
        "program T; begin
      const Get: function(): integer := function(): integer begin return 1; end function;
      Get(); end.",
    );
}

#[test]
fn intrinsic_calls_require_consumption_and_procedures_remain_valid() {
    unused("program T; uses Std.Math; begin Abs(-1); end.");
    unused("program T; uses Std.Fs; begin Std.Fs.DeleteFile('missing'); end.");
    check_ok(
        "program T; uses Std.Console, Std.Tasks;
      function Value(): integer; begin return 1; end function;
      procedure Work(); begin WriteLn('work'); end procedure;
      begin Work(); WriteLn('hello'); go Work(); go Value();
      const Job: task of integer := go Value(); discard Wait(Job); end.",
    );
}

#[test]
fn consumption_forms_and_explicit_discard_remain_valid() {
    check_ok(
        "program T;
      function Value(): integer; begin return 1; end function;
      function Fallible(): result of (integer, string); begin return Ok(1); end function;
      procedure Consume(Value: integer); begin end procedure;
      function Forward(): result of (integer, string);
      begin return Ok(try Fallible()); end function;
      begin const A: integer := Value(); var B: integer := 0;
      B := Value(); Consume(Value()); discard Value(); discard Fallible();
      case Fallible() of when Ok(const V): Consume(V); when Error(const E): discard E; end case;
      end.",
    );
}

#[test]
fn invalid_calls_keep_their_original_diagnostic_without_unused_result_cascade() {
    for source in [
        "program T; begin Missing(); end.",
        "program T; function Value(N: integer): integer; begin return N; end function;
         begin Value(); end.",
        "program T; function Value(N: integer): integer; begin return N; end function;
         begin Value('wrong'); end.",
    ] {
        let errors = check_errors(source);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert!(
            errors
                .iter()
                .all(|error| error.code != SEMA_UNUSED_FUNCTION_RESULT)
        );
    }
}
