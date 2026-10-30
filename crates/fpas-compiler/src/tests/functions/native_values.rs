//! Execute native references through ordinary callable and debugger paths.

use super::super::{assert_succeeds, parse_ok};

#[test]
fn native_callable_mutation_preserves_original_computed_receiver_indices() {
    assert_succeeds(
        r#"
program IndexedMutation;
type Row = record Values: array of integer; end record;
var Rows: array of Row := [Row(Values := [1, 2])];
var Visits: integer := 0;
function Index(): integer;
begin Visits := Visits + 1; return 1; end function;
function Value(): integer;
begin
  if Visits <> 1 then panic('receiver must precede argument'); end if;
  return 3;
end function;
begin
  Rows[Index() - 1].Values.PushInPlace(Value());
  if Visits <> 1 then panic('push index evaluated twice'); end if;
  const Last := Rows[Index() - 1].Values.PopInPlace();
  if Last <> 3 or Visits <> 2 then panic('pop index'); end if;
  Rows[Index() - 1].Values.MapInPlace(function(N: integer): integer begin return N * 2; end function);
  if Visits <> 3 or Rows[0].Values[1] <> 4 then panic('map index'); end if;
  Rows[Index() - 1].Values.ReverseInPlace();
  if Visits <> 4 or Rows[0].Values[0] <> 4 then panic('reverse index'); end if;
end.
"#,
    );
}

#[test]
fn native_callable_values_execute_with_context_aliases_and_value_receivers() {
    for source in [
        include_str!("../../../../../tests/stdlib/fluent/native_callable_routines_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_receivers_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_tasks_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_gc_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_constructors_test.fpas"),
    ] {
        assert_succeeds(source);
    }
}

#[test]
fn native_callable_values_preserve_ordinary_and_seeded_debugger_execution() {
    use fpas_runtime::execution::{DebugRunResult, DebugSession};
    for source in [
        include_str!("../../../../../tests/stdlib/fluent/native_callable_receivers_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_tasks_test.fpas"),
        include_str!("../../../../../tests/stdlib/fluent/native_callable_constructors_test.fpas"),
    ] {
        let program = parse_ok(source);
        let image = crate::compile(&program).unwrap();
        for seed in [None, Some(17)] {
            let mut session = DebugSession::with_limits_and_schedule(
                image.clone(),
                vec![],
                Default::default(),
                Default::default(),
                seed,
            )
            .unwrap();
            assert!(matches!(
                session.continue_execution().unwrap(),
                DebugRunResult::Terminated(_)
            ));
        }
    }
}

#[test]
fn native_callable_values_link_independently_compiled_exports() {
    use super::super::compiled_unit;
    let unit = compiled_unit(
        r#"
unit NativeValues;
uses Std.Math as Math;
public function Absolute(): function(N: integer): integer;
begin return Math.Abs; end function;
public function Length(Text: string): function(): integer;
begin return Text.Length; end function;
public function Wrap(): function(Value: integer): option of integer;
begin return Some; end function;
end unit;
"#,
        &[],
    );
    let interfaces = [unit.interface];
    let root = crate::compile_program_object_with_support(
        &parse_ok(
            r#"
program Consumer;
uses NativeValues as Native;
begin
  const Abs := Native.Absolute();
  if Abs(-7) <> 7 then panic('linked intrinsic'); end if;
  const Length := Native.Length('🙂ab');
  if Length() <> 3 then panic('linked receiver'); end if;
  const Wrap := Native.Wrap();
  if Wrap(8).Unwrap() <> 8 then panic('linked constructor'); end if;
end.
"#,
        ),
        &interfaces,
        &interfaces,
    )
    .unwrap();
    let image = fpas_linker::link_objects(&[unit.object], &root).unwrap();
    fpas_runtime::execution::Vm::new(image).run().unwrap();
}

#[test]
fn native_callable_values_specialize_inside_generic_routines() {
    assert_succeeds(
        r#"
program GenericNativeValues;
function Length<T>(Items: array of T): function(): integer;
begin return Items.Length; end function;
begin
  const Integers := Length([1, 2]);
  const Strings := Length(['a', 'b', 'c']);
  if Integers() <> 2 or Strings() <> 3 then panic('generic binding'); end if;
end.
"#,
    );
}
