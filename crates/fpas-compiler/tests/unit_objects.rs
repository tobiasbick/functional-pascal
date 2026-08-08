#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler integration fixtures use direct assertions for diagnostic clarity"
)]

use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_compiler::{
    compile_program_object, compile_register_program_object_with_support,
    compile_register_unit_object, compile_unit_object,
};
use fpas_linker::{link_objects, link_register_objects};
use fpas_unit::interface::InterfaceType;
use fpas_unit::object::{
    ChunkConstant as ObjectConstant, ChunkDefinitionKind as DefinitionKind,
    ChunkImport as ObjectImport, ChunkLocation as ObjectLocation, ChunkObject as RelocatableObject,
    collect_chunk_relocations as collect_relocations,
};

mod common;

use common::{parse_unit, run_zero_arity};

#[test]
fn register_unit_objects_link_transitive_calls_and_run_initializers() {
    let dependency = parse_unit(
        "unit Demo.Base;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency =
        compile_register_unit_object(&dependency, &[]).expect("register dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Base;
         public function Run(): integer;
         begin return AddOne(41) end;",
    );
    let consumer =
        compile_register_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
            .expect("register consumer compilation");
    let (program, diagnostics) = fpas_parser::parse(
        "program Demo;
         uses Demo.Consumer, Std.Console;
         begin Std.Console.WriteLn(Run()) end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let interfaces = [dependency.interface.clone(), consumer.interface.clone()];
    let program = compile_register_program_object_with_support(
        &program,
        std::slice::from_ref(&interfaces[1]),
        &interfaces,
    )
    .expect("register program compilation");
    let executable = link_register_objects(&[dependency.object, consumer.object], &program)
        .expect("register object linking");
    let mut vm = fpas_vm::RegisterVm::new(executable);
    vm.run().expect("register VM execution");

    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn register_unit_objects_relocate_imported_globals_records_and_enums() {
    let dependency = parse_unit(
        "unit Demo.Model;
         public mutable var Offset: integer := 1;
         public type Point = record
           public Value: integer;
         end;
         public type Choice = enum
           Present(Value: integer);
           Absent;
         end;",
    );
    let dependency =
        compile_register_unit_object(&dependency, &[]).expect("register model compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Model;
         public function Run(): integer;
         begin
           var Item: Point := record Value := Offset + 40; end;
           var Selected: Choice := Choice.Present(Item.Value + 1);
           mutable var Number: integer := 0;
           case Selected of
             Choice.Present(Value): Number := Value;
             Choice.Absent: Number := 0
           end;
           return Number
         end;",
    );
    let consumer =
        compile_register_unit_object(&consumer, std::slice::from_ref(&dependency.interface))
            .expect("register aggregate consumer compilation");
    let (program, diagnostics) = fpas_parser::parse(
        "program Demo;
         uses Demo.Consumer, Std.Console;
         begin Std.Console.WriteLn(Run()) end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let interfaces = [dependency.interface.clone(), consumer.interface.clone()];
    let program = compile_register_program_object_with_support(
        &program,
        std::slice::from_ref(&interfaces[1]),
        &interfaces,
    )
    .expect("register aggregate program compilation");
    let executable = link_register_objects(&[dependency.object, consumer.object], &program)
        .expect("register aggregate object linking");
    let mut vm = fpas_vm::RegisterVm::new(executable);
    vm.run().expect("register aggregate VM execution");

    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn independently_compiled_units_link_and_run_without_dependency_asts() {
    let dependency = parse_unit(
        "unit Demo.Base;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency_compiled =
        compile_unit_object(&dependency, &[]).expect("dependency compilation");

    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Base;
         public function Run(): integer;
         begin return AddOne(41) end;",
    );
    let consumer_compiled = compile_unit_object(
        &consumer,
        std::slice::from_ref(&dependency_compiled.interface),
    )
    .expect("consumer compilation");

    let code = vec![Op::Call(0, 0), Op::PrintLn, Op::Halt];
    let program = RelocatableObject {
        owner: "demo.program".to_string(),
        constants: vec![ObjectConstant::String("demo.consumer.run".to_string())],
        locations: vec![
            ObjectLocation {
                line: 1,
                column: 1,
                source_id: 0,
            };
            code.len()
        ],
        functions: BTreeMap::new(),
        definitions: Vec::new(),
        imports: vec![ObjectImport {
            name: "demo.consumer.run".to_string(),
            kind: DefinitionKind::Callable,
        }],
        relocations: collect_relocations(&code),
        code,
    };

    let chunk = link_objects(
        &[dependency_compiled.object, consumer_compiled.object],
        &program,
    )
    .expect("object linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");
    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn independently_compiled_program_uses_unit_interface_and_object() {
    let dependency = parse_unit(
        "unit Demo.Base;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency_compiled =
        compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let (program, diagnostics) = fpas_parser::parse(
        "program Demo;
         uses Demo.Base, Std.Console;
         begin Std.Console.WriteLn(AddOne(41)) end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let program_object = compile_program_object(
        &program,
        std::slice::from_ref(&dependency_compiled.interface),
    )
    .expect("program compilation");
    let chunk = link_objects(&[dependency_compiled.object], &program_object).expect("linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");
    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn program_routine_shadows_imported_enum_constructor_during_lowering() {
    let dependency = parse_unit(
        "unit Demo.Events;
         public type Input = enum
           Pointer(Value: integer);
         end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let (program, diagnostics) = fpas_parser::parse(
        "program Demo;
         uses Demo.Events, Std.Console;
         function Pointer(Value: integer): integer;
         begin return Value + 1 end;
         begin Std.Console.WriteLn(Pointer(41)) end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let program_object =
        compile_program_object(&program, std::slice::from_ref(&dependency.interface))
            .expect("program compilation");
    let chunk = link_objects(&[dependency.object], &program_object).expect("linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");

    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn imported_record_defaults_are_compiled_from_the_unit_interface() {
    let dependency = parse_unit(
        "unit Demo.Config;
         public type Config = record
           public Host: string := 'localhost';
           public Port: integer := 8080;
         end;",
    );
    let dependency_compiled =
        compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let (program, diagnostics) = fpas_parser::parse(
        "program Demo;
         uses Demo.Config, Std.Console;
         begin
           var Value: Config := record end;
           Std.Console.WriteLn(Value.Host);
           Std.Console.WriteLn(Value.Port)
         end.",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let program_object = compile_program_object(
        &program,
        std::slice::from_ref(&dependency_compiled.interface),
    )
    .expect("program compilation");
    let chunk = link_objects(&[dependency_compiled.object], &program_object).expect("linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");

    assert_eq!(vm.output().lines, ["localhost", "8080"]);
}

#[test]
fn imported_function_can_be_passed_as_a_callable_value() {
    let dependency = parse_unit(
        "unit Demo.Base;
         public function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Base;
         function Apply(F: function(X: integer): integer; Value: integer): integer;
         begin return F(Value) end;
         public function Run(): integer;
         begin return Apply(AddOne, 41) end;",
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
fn nested_unit_routines_resolve_to_their_qualified_object_entries() {
    let unit = parse_unit(
        "unit Demo.Nested;
         public function Run(): integer;
         function AddOne(Value: integer): integer;
         begin return Value + 1 end;
         begin return AddOne(41) end;",
    );
    let unit = compile_unit_object(&unit, &[]).expect("unit compilation");

    assert_eq!(run_zero_arity(vec![unit.object], "demo.nested.run"), ["42"]);
}

#[test]
fn concurrent_private_record_method_chains_resolve_to_qualified_unit_entries() {
    let unit = parse_unit(
        "unit Demo.RecordMethods;
         uses Std.Task;
         type Counter = record
           Value: integer;
           function Increment(Self: Counter): Counter;
           begin return record Value := Self.Value + 1; end end;
           function Add(Self: Counter; Other: Counter): Counter;
           begin return record Value := Self.Value + Other.Value; end end;
         end;
         function Compute(): integer;
         begin
           var Left: Counter := record Value := 40; end;
           var Right: Counter := record Value := 1; end;
           return Left.Increment().Add(Right).Value
         end;
         public function Run(): integer;
         begin
           var Work: task := go Compute();
           return Wait(Work)
         end;",
    );
    let unit = compile_unit_object(&unit, &[]).expect("unit compilation");

    assert_eq!(
        run_zero_arity(vec![unit.object], "demo.recordmethods.run"),
        ["42"]
    );
}

#[test]
fn unit_object_rejects_implicit_enum_backing_value_after_i64_max() {
    let unit = parse_unit(
        "unit Demo.EnumLimit;
         public type Limit = enum
           Last = 9223372036854775807;
           Overflow;
         end;",
    );

    let errors = match compile_unit_object(&unit, &[]) {
        Ok(_) => panic!("implicit enum backing value after i64::MAX must fail"),
        Err(errors) => errors,
    };

    assert!(errors.iter().any(|error| {
        error.code == fpas_diagnostics::codes::SEMA_ENUM_BACKING_VALUE_EXHAUSTED
            && error.message.contains("Limit.Overflow")
    }));
}

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
