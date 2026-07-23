#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler integration fixtures use direct assertions for diagnostic clarity"
)]

use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_compiler::{compile_program_object, compile_unit_object};
use fpas_linker::link_objects;
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_unit::object::{
    DefinitionKind, ObjectConstant, ObjectImport, ObjectLocation, RelocatableObject,
    collect_relocations,
};

fn parse_unit(source: &str) -> fpas_parser::Unit {
    let (parsed, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must be a unit");
    };
    unit
}

fn run_zero_arity(objects: Vec<RelocatableObject>, callable: &str) -> Vec<String> {
    let code = vec![Op::Call(0, 0), Op::PrintLn, Op::Halt];
    let program = RelocatableObject {
        owner: "demo.program".to_string(),
        constants: vec![ObjectConstant::String(callable.to_string())],
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
            name: callable.to_string(),
            kind: DefinitionKind::Callable,
        }],
        relocations: collect_relocations(&code),
        code,
    };
    let chunk = link_objects(&objects, &program).expect("object linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");
    vm.output().lines.clone()
}

#[test]
fn independently_compiled_units_link_and_run_without_dependency_asts() {
    let dependency = parse_unit(
        "unit Demo.Base;
         function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency_compiled =
        compile_unit_object(&dependency, &[]).expect("dependency compilation");

    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Base;
         function Run(): integer;
         begin return AddOne(41) end;",
    );
    let consumer_compiled =
        compile_unit_object(&consumer, &[dependency_compiled.interface.clone()])
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
         function AddOne(Value: integer): integer;
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
    let program_object = compile_program_object(&program, &[dependency_compiled.interface.clone()])
        .expect("program compilation");
    let chunk = link_objects(&[dependency_compiled.object], &program_object).expect("linking");
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("linked VM execution");
    assert_eq!(vm.output().lines, ["42"]);
}

#[test]
fn imported_record_defaults_are_compiled_from_the_unit_interface() {
    let dependency = parse_unit(
        "unit Demo.Config;
         type Config = record
           Host: string := 'localhost';
           Port: integer := 8080;
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

    let program_object = compile_program_object(&program, &[dependency_compiled.interface.clone()])
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
         function AddOne(Value: integer): integer;
         begin return Value + 1 end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Base;
         function Apply(F: function(X: integer): integer; Value: integer): integer;
         begin return F(Value) end;
         function Run(): integer;
         begin return Apply(AddOne, 41) end;",
    );
    let consumer = compile_unit_object(&consumer, &[dependency.interface.clone()])
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
         function Run(): integer;
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
         private type Counter = record
           Value: integer;
           function Increment(Self: Counter): Counter;
           begin return record Value := Self.Value + 1; end end;
           function Add(Self: Counter; Other: Counter): Counter;
           begin return record Value := Self.Value + Other.Value; end end;
         end;
         private function Compute(): integer;
         begin
           var Left: Counter := record Value := 40; end;
           var Right: Counter := record Value := 1; end;
           return Left.Increment().Add(Right).Value
         end;
         function Run(): integer;
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
fn local_variables_shadow_imported_enum_variant_aliases_during_assignment() {
    let dependency = parse_unit(
        "unit Demo.Policy;
         type
           Policy = enum
             Preferred(Value: integer);
           end;",
    );
    let dependency = compile_unit_object(&dependency, &[]).expect("dependency compilation");
    let consumer = parse_unit(
        "unit Demo.Consumer;
         uses Demo.Policy, Std.Array;
         function Run(): integer;
         begin
           mutable var Preferred: array of integer := [];
           Preferred := Std.Array.Concat(Preferred, [42]);
           return Preferred[0]
         end;",
    );
    let consumer = compile_unit_object(&consumer, &[dependency.interface.clone()])
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
         function Run(): integer;
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
