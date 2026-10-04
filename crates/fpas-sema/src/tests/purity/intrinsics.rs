use super::rejects;
use crate::tests::check_ok;

#[test]
fn dictionary_reduction_rejects_effectful_callbacks_and_mutable_captures() {
    for modifier in ["", "pure "] {
        rejects(&format!(
            "program Main; uses Std.Dictionaries as Dictionaries;
            begin var Current: dict of (string, integer) := ['A': 1];
              discard Dictionaries.Reduce(Current, 0,
                {modifier}function(Acc: integer; Key: string; Value: integer): integer
                begin Current['B'] := 2; return Acc + Value; end function);
            end program;"
        ));
    }
}

#[test]
fn option_and_result_projections_validate_containers_and_fallbacks() {
    for expression in [
        "Options.IsSome(1)",
        "Options.IsNone(Result.Ok(1))",
        "Results.IsOk(Option.Some(1))",
        "Results.IsError(false)",
        "Options.UnwrapOr(Option.Some(1), 'wrong')",
        "Results.UnwrapOr(Result.Ok(1), false)",
    ] {
        rejects(&format!(
            "program Main; uses Std.Options as Options; uses Std.Results as Results;
            begin discard {expression}; end program;"
        ));
    }
    check_ok(
        "program Main; uses Std.Options as Options; uses Std.Results as Results;
        begin const Missing: Option of (integer) := Option.None;
          const Failure: Result of (integer, string) := Result.Error('missing');
          discard Options.IsNone(Missing); discard Results.IsError(Failure);
          discard Options.UnwrapOr(Missing, 42); discard Results.UnwrapOr(Failure, 42);
        end program;",
    );
}

#[test]
fn pure_intrinsics_accept_contextually_instantiated_callbacks_and_value_data() {
    check_ok("program Main;
        uses Std.Arrays as Arrays; uses Std.Options as Options; uses Std.Results as Results;
        uses Std.Math as Math; uses Std.Str as Str;
        pure function Identity of (T)(Value: T): T; begin return Value; end function;
        pure function Size of (T)(Values: array of (T)): integer; begin return Arrays.Length(Values); end function;
        pure function Calculate(): integer;
        begin
          const Values := Arrays.Map([1, 2], Identity);
          const Optional := Options.Map(Option.Some(3), Identity);
          const Success: Result of (integer, string) := Result.Ok(4);
          const Mapped := Results.Map(Success, Identity);
          return Size(Values) + Options.Unwrap(Optional) + Results.Unwrap(Mapped) + Math.Abs(-1);
        end function;
        begin discard Calculate(); discard Str.Format('{0}', 42); end program;");
}

#[test]
fn intrinsic_callbacks_validate_arity_modes_payloads_and_container_results() {
    for call in [
        "Options.Map(Option.Some(1), pure function(Value: string): integer begin return 1; end function)",
        "Options.Map(Option.Some(1), pure function(): integer begin return 1; end function)",
        "Options.AndThen(Option.Some(1), pure function(Value: integer): integer begin return Value; end function)",
        "Options.OrElse(Option.Some(1), pure function(): Option of (string) begin return Option.Some('x'); end function)",
        "Results.AndThen(Input, pure function(Value: integer): Result of (integer, boolean) begin return Result.Ok(Value); end function)",
        "Results.OrElse(Input, pure function(Value: string): Result of (string, string) begin return Result.Ok(Value); end function)",
        "Arrays.Map([1], function(Value: integer): integer begin return Value; end function)",
    ] {
        rejects(&format!("program Main;
            uses Std.Options as Options; uses Std.Results as Results; uses Std.Arrays as Arrays;
            begin const Input: Result of (integer, string) := Result.Ok(1); discard {call}; end program;"));
    }
    rejects(
        "program Main; uses Std.Arrays as Arrays;
        procedure Change(var Value: integer); begin Value := 2; end procedure;
        begin Arrays.ForEach([1], Change); end program;",
    );
}

#[test]
fn resource_arguments_and_effectful_intrinsics_cannot_cross_pure_boundaries() {
    rejects("program Main; uses Std.Str as Str;
        begin const Values: array of (channel of (integer)) := []; discard Str.Format('{0}', Values); end program;");
    rejects("program Main; uses Std.Arrays as Arrays;
        begin const Values: array of (procedure()) := []; discard Arrays.Length(Values); end program;");
    rejects(
        "program Main; uses Std.Time as Time;
        pure function Clock(): integer; begin return Time.TimestampMillis(); end function;
        begin null; end program;",
    );
    check_ok("program Main; uses Std.Arrays as Arrays;
        begin var Count := 0; Arrays.ForEach([1, 2], procedure(Value: integer) begin Count := Count + Value; end procedure); end program;");
}
