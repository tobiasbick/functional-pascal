use super::rejects;
use crate::tests::check_ok;

#[test]
fn pure_functions_support_local_mutation_recursion_and_immutable_captures() {
    check_ok("program Main;
        pure function Identity of (T)(Value: T): T; begin return Value; end function;
        pure function Factorial(Value: integer): integer;
        begin
          var ResultValue := 1;
          for Index: integer := 1 to Value do ResultValue := ResultValue * Index; end for;
          return ResultValue;
        end function;
        pure function Factory(Base: integer): pure function(Value: integer): integer;
        begin return pure function(Value: integer): integer begin return Base + Value; end function; end function;
        begin
          const Callback: pure function(Input: integer): integer := Identity;
          const Ordinary: function(Input: integer): integer := Callback;
          discard Factory(Factorial(3))(Identity(4)); discard Ordinary(1);
        end program;");
}

#[test]
fn pure_capability_cannot_be_recovered_after_forgetting_it() {
    for expression in ["Ordinary", "Stored", "Factory()"] {
        rejects(&format!(
            "program Main;
            pure function Original(): integer; begin return 1; end function;
            function Ordinary(): integer; begin return 1; end function;
            function Factory(): function(): integer; begin return Original; end function;
            begin
              const Stored: function(): integer := Original;
              const Required: pure function(): integer := {expression};
            end program;"
        ));
    }
}

#[test]
fn pure_signatures_reject_resource_and_ordinary_callable_components() {
    for ty in [
        "procedure()",
        "function(): integer",
        "Option of (procedure())",
        "array of (channel of (integer))",
        "task of (integer)",
    ] {
        rejects(&format!(
            "program Main; pure function Bad(Value: {ty}): integer; begin return 1; end function; begin null; end program;"
        ));
    }
    rejects(
        "program Main; pure function Bad(var Value: integer): integer; begin return Value; end function; begin null; end program;",
    );
    rejects(
        "program Main; pure function Bad(): Option of (procedure()); begin return Option.None; end function; begin null; end program;",
    );
    rejects("program Main;
        pure function Identity of (T)(Value: T): T; begin return Value; end function;
        begin const Callback: function(Value: channel of (integer)): channel of (integer) := Identity; end program;");
}

#[test]
fn ordinary_generic_signatures_carry_explicit_pure_callback_requirements() {
    check_ok(
        "program Main;
        type CallbackBox of (T) = record Callback: pure function(Value: T): T; end record;
        pure function Identity of (T)(Value: T): T; begin return Value; end function;
        function Apply of (T)(Value: T; Callback: pure function(Input: T): T): T;
        begin
          const Copy := pure function(Input: T): T begin return Value; end function;
          return Callback(Copy(Value));
        end function;
        begin
          const Boxed: CallbackBox of (integer) := CallbackBox(Callback := Identity);
          discard Apply(42, Boxed.Callback);
        end program;",
    );
    rejects("program Main;
        type CallbackBox of (T) = record Callback: pure function(Value: T): T; end record;
        begin const Boxed: Option of (CallbackBox of (procedure())) := Option.None; discard Boxed; end program;");
    rejects(
        "program Main;
        function Factory of (T)(Value: T): function(): T;
        begin return pure function(): T begin return Value; end function; end function;
        begin null; end program;",
    );
}

#[test]
fn callable_variance_and_decisions_preserve_only_available_guarantees() {
    check_ok(
        "program Main;
        pure function Clean(): integer; begin return 1; end function;
        function Ordinary(): integer; begin return 1; end function;
        function AcceptAny(F: function(): integer): integer; begin return F(); end function;
        begin
          const One := if true then Clean else Ordinary end if;
          const Two := if true then Ordinary else Clean end if;
          const Safe: function(F: pure function(): integer): integer := AcceptAny;
          discard One(); discard Two(); discard Safe(Clean);
        end program;",
    );
    rejects(
        "program Main;
        function NeedsPure(F: pure function(): integer): integer; begin return F(); end function;
        begin const Unsafe: function(F: function(): integer): integer := NeedsPure; end program;",
    );
    rejects(
        "program Main;
        procedure Change(var F: function(): integer); begin null; end procedure;
        begin const Unsafe: procedure(var F: pure function(): integer) := Change; end program;",
    );
}

#[test]
fn nominal_arguments_cannot_reverse_a_callback_requirement() {
    rejects("program Main;
        type Box of (T) = record Callback: function(Value: T): integer; end record;
        function NeedsPure(Value: pure function(): integer): integer; begin return Value(); end function;
        function Ordinary(): integer; begin return 42; end function;
        begin
          const Original: Box of (pure function(): integer) := Box(Callback := NeedsPure);
          const Invalid: Box of (function(): integer) := Original;
          discard Invalid.Callback(Ordinary);
        end program;");
}
