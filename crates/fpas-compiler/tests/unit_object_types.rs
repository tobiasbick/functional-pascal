#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler integration fixtures use direct assertions for diagnostic clarity"
)]

use fpas_compiler::compile_unit_object;
use fpas_unit::interface::InterfaceType;

mod common;

use common::{parse_unit, run_zero_arity};

#[test]
fn unit_interface_preserves_explicit_restart_after_i64_max() {
    let unit = parse_unit(
        "unit Demo.EnumRestart;
         public type Limit = enum
           Last = 9223372036854775807;
           Restart = 0;
           Next;
         end;",
    );
    let compiled = compile_unit_object(&unit, &[]).expect("unit compilation");
    let symbol = compiled
        .interface
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Limit")
        .expect("exported enum symbol");
    let InterfaceType::Enum(enum_type) = &symbol.ty else {
        panic!("Limit must be exported as an enum");
    };
    let backing_values: Vec<_> = enum_type
        .variants
        .iter()
        .map(|variant| variant.backing_value)
        .collect();

    assert_eq!(backing_values, [Some(i64::MAX), Some(0), Some(1)]);
}

#[test]
fn local_variables_shadow_imported_enum_variant_aliases_during_assignment() {
    let dependency = parse_unit(
        "unit Demo.Policy;
         public type
           Policy = enum
             Preferred(Value: integer);
           end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Policy, Std.Array;
         public function Run(): integer;
         begin
           mutable var Preferred: array of integer := [];
           Preferred := Std.Array.Concat(Preferred, [42]);
           return Preferred[0]
         end;",
    );
    let consumer = compile_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
        .expect("consumer compilation");

    assert_eq!(
        run_zero_arity(
            vec![dependency.object, consumer.object],
            "demo.consumer.run"
        ),
        ["42"]
    );
}

#[test]
fn unit_owned_data_enum_patterns_use_the_qualified_runtime_identity() {
    let unit = parse_unit(
        "unit Demo.Shape;
         type
           Shape = enum
             Point(Value: integer);
             Empty;
           end;
         public function Run(): integer;
         begin
           var Value: Shape := Shape.Point(42);
           case Value of
             Shape.Point(Number):
             begin
               return Number
             end;
             Shape.Empty:
             begin
               return 0
             end
           end
         end;",
    );
    let unit = compile_unit_object(&unit, &[]).expect("unit compilation");

    assert_eq!(run_zero_arity(vec![unit.object], "demo.shape.run"), ["42"]);
}
