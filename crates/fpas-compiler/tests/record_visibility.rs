#![allow(
    clippy::expect_used,
    reason = "compiler integration fixtures use expect for focused assertions"
)]

mod common;

use common::{parse_unit, run_zero_arity};
use fpas_compiler::{CompiledUnitObject, compile_unit_object};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{SEMA_PRIVATE_RECORD_MEMBER, SEMA_TYPE_MISMATCH};

fn compile_counter_unit() -> CompiledUnitObject {
    let unit = parse_unit(
        "unit Demo.Counter;
         type
           Counter = record
             private Value: integer := 0;
             Step: integer := 1;

             private static function FromValue(Value: integer): Counter;
             begin
               return record Value := Value; Step := 1; end
             end;

             static function Create(Value: integer): Counter;
             begin
               return Counter.FromValue(Value)
             end;

             private function Hidden(Self: Counter): integer;
             begin
               return Self.Value
             end;

             function Current(Self: Counter): integer;
             begin
               return Self.Hidden()
             end;
           end;

         function LocalCheck(): integer;
         begin
           mutable var CounterValue: Counter := record Value := 40; Step := 2; end;
           CounterValue.Value := CounterValue.Value + 1;
           return CounterValue.Hidden() + CounterValue.Step - 1
         end;",
    );
    compile_unit_object(&unit, &[]).expect("counter unit compilation")
}

fn assert_consumer_error(source_body: &str, expected: DiagnosticCode) {
    let dependency = compile_counter_unit();
    let consumer = parse_unit(&format!(
        "unit Demo.Consumer;
         uses Demo.Counter;
         function Run(): integer;
         begin
           {source_body}
         end;"
    ));
    let diagnostics = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .err()
        .expect("consumer must be rejected");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == expected),
        "{diagnostics:#?}"
    );
}

#[test]
fn declaring_unit_can_use_private_record_members() {
    let unit = compile_counter_unit();

    assert_eq!(
        run_zero_arity(vec![unit.object], "demo.counter.localcheck"),
        ["42"]
    );
}

#[test]
fn importing_unit_can_use_public_record_members() {
    let dependency = compile_counter_unit();
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Counter;
         function Run(): integer;
         begin
           var Value: Counter := Counter.Create(40);
           return Value.Current() + Value.Step
         end;",
    );
    let consumer = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .expect("public record members must compile");

    assert_eq!(
        run_zero_arity(
            vec![dependency.object, consumer.object],
            "demo.consumer.run"
        ),
        ["41"]
    );
}

#[test]
fn importing_unit_cannot_read_private_field() {
    assert_consumer_error(
        "var Value: Counter := Counter.Create(1);
         return Value.Value",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
}

#[test]
fn importing_unit_cannot_write_private_field() {
    assert_consumer_error(
        "mutable var Value: Counter := Counter.Create(1);
         Value.Value := 2;
         return 0",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
}

#[test]
fn importing_unit_cannot_call_private_instance_or_static_routines() {
    assert_consumer_error(
        "var Value: Counter := Counter.Create(1);
         return Value.Hidden()",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
    assert_consumer_error(
        "var Value: Counter := Counter.FromValue(1);
         return Value.Current()",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
}

#[test]
fn importing_unit_cannot_construct_record_with_private_fields() {
    assert_consumer_error(
        "var Value: Counter := record end;
         return Value.Current()",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
    assert_consumer_error(
        "var Value: Counter := record Value := 1; Step := 2; end;
         return Value.Current()",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
}

#[test]
fn importing_unit_cannot_update_private_field() {
    assert_consumer_error(
        "var Value: Counter := Counter.Create(1);
         var Changed: Counter := Value with Value := 2; end;
         return Changed.Current()",
        SEMA_PRIVATE_RECORD_MEMBER,
    );
}

#[test]
fn structurally_equivalent_literal_cannot_bypass_private_construction() {
    let dependency = compile_counter_unit();
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Counter;
         type ForgedCounter = record Value: integer; Step: integer; end;
         function Run(): integer;
         begin
           var Forged: ForgedCounter := record Value := 1; Step := 2; end;
           var Value: Counter := Forged;
           return Value.Current()
         end;",
    );
    let diagnostics = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .err()
        .expect("structural conversion must be rejected");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SEMA_TYPE_MISMATCH),
        "{diagnostics:#?}"
    );
}

#[test]
fn private_record_cannot_be_converted_to_structurally_equivalent_public_record() {
    let dependency = compile_counter_unit();
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Counter;
         type ForgedCounter = record Value: integer; Step: integer; end;
         function Run(): integer;
         begin
           var Value: Counter := Counter.Create(1);
           var Forged: ForgedCounter := Value;
           return Forged.Value
         end;",
    );
    let diagnostics = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .err()
        .expect("private layout exposure must be rejected");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SEMA_TYPE_MISMATCH),
        "{diagnostics:#?}"
    );
}

#[test]
fn private_routine_without_private_fields_does_not_block_public_literal() {
    let dependency = parse_unit(
        "unit Demo.OpenRecord;
         type
           OpenRecord = record
             Value: integer;
             private function Hidden(Self: OpenRecord): integer;
             begin return Self.Value end;
           end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.OpenRecord;
         function Run(): integer;
         begin
           var Value: OpenRecord := record Value := 42; end;
           return Value.Value
         end;",
    );
    let consumer = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .expect("public-field literal must compile");

    assert_eq!(
        run_zero_arity(
            vec![dependency.object, consumer.object],
            "demo.consumer.run"
        ),
        ["42"]
    );
}
