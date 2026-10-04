//! Pure functions retain runtime callable and generic representation.

use crate::tests::assert_succeeds;

#[test]
fn pure_generic_functions_closures_and_intrinsic_callbacks_execute() {
    assert_succeeds("program Main;
        uses Std.Arrays as Arrays; uses Std.Options as Options;
        pure function Identity of (T)(Value: T): T; begin return Value; end function;
        pure function Factory(Base: integer): pure function(Value: integer): integer;
        begin return pure function(Value: integer): integer begin return Base + Value; end function; end function;
        type Settings = record Count: integer := Identity(42); Action: Option of (procedure()) := Option.None; end record;
        begin
          const Values := Arrays.Map([20, 21], Identity);
          const Mapped := Arrays.Map(Values, Factory(21));
          const Value := Options.Map(Option.Some(42), Identity);
          if Mapped[1] <> 42 then panic('pure map'); end if;
          if Options.Unwrap(Value) <> 42 then panic('pure option'); end if;
          const Config := Settings();
          if Config.Count <> 42 then panic('pure default'); end if;
        end program;");
}
