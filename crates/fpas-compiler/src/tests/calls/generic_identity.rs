//! Generic substitution preserves captured outer types through erased runtime calls.
//!
//! **Documentation:** `docs/pascal/language/functions/generic-routines.md`.

use crate::tests::assert_succeeds;

#[test]
fn generic_parameter_identity_preserves_nested_returns_and_closures() {
    assert_succeeds(
        "program Main;
        function Outer of (T)(Value: T): function(): T;
          function Preserve(Captured: T): T; begin return Captured; end function;
          function Inner of (t)(Other: t): t; begin discard Preserve(Value); return Other; end function;
        begin return function(): T begin discard Inner(1); return Preserve(Value); end function; end function;
        begin
          const Text := Outer('text');
          const Number := Outer(41);
          if Text() <> 'text' then panic('captured text type'); end if;
          if Number() + 1 <> 42 then panic('captured numeric type'); end if;
        end program;",
    );
}

#[test]
fn generic_call_results_retain_inferred_types_in_scalar_operations() {
    assert_succeeds(
        "program Main;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        begin
          if Identity(41) + 1 <> 42 then panic('generic addition'); end if;
          if Identity(42) div 2 <> 21 then panic('generic integer division'); end if;
          if -Identity(42) <> -42 then panic('generic negation'); end if;
          if Identity('a') + Identity('b') <> 'ab' then panic('generic concatenation'); end if;
          if not Identity(true) then panic('generic boolean'); end if;
          if Identity(21) * 2.0 <> 42.0 then panic('generic promotion'); end if;
        end program;",
    );
}

#[test]
fn generic_callable_values_execute_after_contextual_instantiation() {
    assert_succeeds(
        "program Main;
        type Handler = function(Value: integer): integer;
        type Box of (T) = record Value: T; end record;
        function Identity of (T)(Value: T): T; begin return Value; end function;
        procedure Ignore of (T)(Value: T); begin discard Value; end procedure;
        function Apply of (T)(Callback: function(Input: T): T; Value: T): T;
        begin return Callback(Value); end function;
        begin
          const Callback: function(Input: integer): integer := Identity;
          const Action: procedure(Input: string) := Ignore;
          const Functions: array of (function(Input: integer): integer) := [Identity];
          const Nested: array of (array of (Handler)) := [[Identity]];
          const Dictionary: dict of (string, Handler) := ['one': Identity];
          const Boxed: Box of (Handler) := Box(Value := Identity);
          const Optional: Option of (Handler) := Option.Some(Identity);
          const Success: Result of (Handler, string) := Result.Ok(Identity);
          Action('text');
          if Callback(41) + 1 <> 42 then panic('stored generic callback'); end if;
          if Functions[0](42) <> 42 then panic('generic callback array'); end if;
          if Nested[0][0](42) <> 42 then panic('nested callback array'); end if;
          if Dictionary['one'](42) <> 42 then panic('callback dictionary'); end if;
          if Boxed.Value(42) <> 42 then panic('callback record'); end if;
          case Optional of
            when Option.Some(const Call): if Call(42) <> 42 then panic('callback option'); end if;
            when Option.None: panic('missing callback');
          end case;
          case Success of
            when Result.Ok(const Call): if Call(42) <> 42 then panic('callback result'); end if;
            when Result.Error(const Message): panic(Message);
          end case;
          if Apply(Identity, 41) + 1 <> 42 then panic('first generic callback'); end if;
          if Apply(Identity, 'text') <> 'text' then panic('text generic callback'); end if;
        end program;",
    );
}
