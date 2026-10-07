use super::*;

mod migration;
mod units;

#[test]
fn loop_const_captures_keep_each_iterations_value_after_the_loop() {
    assert_succeeds("program T; uses Std.Arrays;
        begin var Callbacks: array of function(): integer := [];
        for I: integer := 1 to 3 do
            const Current: integer := I * 10;
            const Callback: function(): integer := function(): integer begin return Current; end function;
            Std.Arrays.Push(Callbacks, Callback);
        end for;
        const First: function(): integer := Callbacks[0];
        const Second: function(): integer := Callbacks[1];
        const Third: function(): integer := Callbacks[2];
        if First() <> 10 or Second() <> 20 or Third() <> 30 then panic('iteration capture'); end if;
        end.");
}

#[test]
fn task_const_preserves_inferred_result_type_at_global_and_local_levels() {
    assert_succeeds(
        "program T; uses Std.Tasks;
        function Work(): integer; begin return 42; end function;
        const GlobalTask: task := go Work();
        begin const LocalTask: task := go Work();
        const First: integer := Wait(GlobalTask); const Second: integer := Wait(LocalTask);
        if First <> 42 or Second <> 42 then panic('task result'); end if; end.",
    );
}

#[test]
fn initializers_follow_reachability_order_and_loop_iterations() {
    assert_succeeds(include_str!(
        "../../../../../tests/runner/computed_const_test.fpas"
    ));
}

#[test]
fn initializer_failure_occurs_when_the_declaration_is_reached() {
    let error = run_program(
        "program T;
        function Fail(): integer; begin panic('const initializer reached'); return 0; end function;
        begin const Value: integer := Fail(); panic('after initializer'); end.",
    )
    .expect_err("initializer must fail");
    assert!(
        error.message.contains("const initializer reached"),
        "{error:?}"
    );
    assert_succeeds(
        "program T;
        function Fail(): integer; begin panic('unreachable'); return 0; end function;
        procedure Stop(); begin return; const Value: integer := Fail(); end procedure;
        begin if false then const Value: integer := Fail(); end if;
        for I: integer := 1 to 0 do const Value: integer := Fail(); end for;
        Stop(); end.",
    );
}

#[test]
fn closures_copy_const_values_and_preserve_shadowed_declarations() {
    assert_succeeds("program T;
        function Make(Value: integer): function(): integer;
        begin const Saved: integer := Value;
        return function(): integer begin return Saved; end function; end function;
        function Nested(): integer;
        function ReadSaved(): integer; begin return Saved; end function;
        begin const Saved: integer := 12; return ReadSaved(); end function;
        begin var Source: integer := 7; const Saved: integer := Source;
        const ReadSaved: function(): integer := function(): integer begin return Saved; end function;
        Source := 99;
        begin const Saved: integer := 11;
            const Inner: function(): integer := function(): integer begin return Saved; end function;
            if Inner() <> 11 then panic('shadow capture'); end if;
        end;
        if ReadSaved() <> 7 then panic('copy capture'); end if;
        if Nested() <> 12 then panic('named capture'); end if;
        const First: function(): integer := Make(21); const Second: function(): integer := Make(34);
        if First() <> 21 or Second() <> 34 then panic('escaped captures'); end if; end.");
}
