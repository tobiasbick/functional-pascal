//! Native mutations obey selected caller storage and ordinary invocation cleanup.

use crate::tests::{assert_succeeds, run_program};

#[test]
fn array_mutations_support_nested_storage_forwarding_and_value_copies() {
    assert_succeeds(
        r#"program ArrayReferences;
uses Std.Arrays as Arrays;
type Box = record Items: array of (integer); end record;
procedure Append(var Items: array of (integer)); begin Arrays.Push(var Items, 3); end procedure;
procedure Add of (T)(var Items: array of (T); Value: T); begin Arrays.Push(var Items, Value); end procedure;
procedure Check();
begin
   var Data: Box := Box(Items := [1, 2]);
  const Copy: Box := Data;
  Append(var Data.Items);
  if Arrays.Pop(var Data.Items) <> 3 then panic('field pop'); end if;
   var Nested: array of (array of (integer)) := [[4]];
  const Old: array of (array of (integer)) := Nested;
  Arrays.Push(var Nested[0], Nested[0][0]);
   var Mapping: dict of (string, array of (integer)) := ['key': [8]];
  Arrays.Push(var Mapping['key'], 9);
  if (Arrays.Pop(var Mapping['key']) <> 9) or (Arrays.Length(Nested[0]) <> 2) or
     (Arrays.Length(Old[0]) <> 1) or (Arrays.Length(Copy.Items) <> 2) then panic('snapshot'); end if;
   var Reals: array of (real) := [];
  Add(var Reals, 42.0);
  if Arrays.Pop(var Reals) <> 42.0 then panic('element type'); end if;
  Add(var Data.Items, 9);
  if Arrays.Pop(var Data.Items) <> 9 then panic('generic integer'); end if;
end procedure;
begin Check(); end program;"#,
    );
}

#[test]
fn array_mutation_reserves_storage_before_later_argument_side_effects() {
    let error = run_program(
        r#"program ArrayReservation;
uses Std.Arrays as Arrays;
 var Data: array of (integer) := [1];
function Later(): integer; begin Data[0] := 9; return 2; end function;
begin Arrays.Push(var Data, Later()); end program;"#,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT
    );
}

#[test]
fn empty_array_pop_reports_the_original_source_call() {
    let error = run_program(
        r#"program Empty; uses Std.Arrays as Arrays;
begin  var A: array of (integer) := [];
  discard Arrays.Pop(var A);
end program;"#,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS
    );
    assert_eq!(error.span.expect("source call span").line(), 3);
}
