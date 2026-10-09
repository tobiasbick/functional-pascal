//! Program headings retain their identity without declaring source symbols.
//!
//! **Documentation:** `docs/pascal/getting-started/first-program.md`

use super::super::parse_ok;
use fpas_bytecode::VerifiedExecutable;
use fpas_unit::object::DefinitionTarget;

fn compile_and_link(
    source: &str,
    declaration: &str,
    target: DefinitionTarget,
) -> VerifiedExecutable {
    let program = parse_ok(source);
    let direct = crate::compile(&program).expect("direct compilation");
    fpas_vm::Vm::new(direct).run().expect("direct execution");
    let object = crate::compile_object(&program).expect("object compilation");
    assert_eq!(object.owner, program.name.to_ascii_lowercase());
    assert!(
        object
            .definitions
            .iter()
            .any(|definition| { definition.name == declaration && definition.target == target })
    );
    let entry = object.entry.expect("program entry");
    assert_ne!(object.functions[entry as usize].name, declaration);
    let encoded = fpas_unit::object::encode_object(&object).expect("encode object");
    let decoded = fpas_unit::object::decode_object(&encoded).expect("decode object");
    assert_eq!(decoded, object);
    let linked = fpas_linker::link_objects(&[], &decoded).expect("link object");
    fpas_vm::Vm::new(linked.clone())
        .run()
        .expect("linked execution");
    linked
}

#[test]
fn program_heading_can_match_a_recursive_function() {
    compile_and_link(
        "program dEmO;
         function Demo(N: integer): integer;
         begin
           if N = 0 then return 1; end if;
           return N * Demo(N - 1);
         end function;
         begin if Demo(4) <> 24 then panic('recursive call'); end if; end.",
        "demo",
        DefinitionTarget::Function(1),
    );
}

#[test]
fn program_heading_can_match_a_procedure() {
    compile_and_link(
        "program Demo;
         var Called: boolean := false;
         procedure Demo(); begin Called := true; end procedure;
         begin Demo(); if not Called then panic('procedure call'); end if; end.",
        "demo",
        DefinitionTarget::Function(1),
    );
}

#[test]
fn program_heading_can_match_an_initialized_global() {
    compile_and_link(
        "program cOuNtEr;
         var Counter: integer := 40 + 1;
         begin Counter := Counter + 1;
           if Counter <> 42 then panic('global initializer'); end if; end.",
        "counter",
        DefinitionTarget::Global(0),
    );
}

#[test]
fn program_heading_can_match_a_record_type() {
    compile_and_link(
        "program pOiNt;
         type Point = record X: integer := 7; end record;
         begin const P: Point := Point();
           if P.X <> 7 then panic('record default'); end if; end.",
        "point",
        DefinitionTarget::Record(0),
    );
}

#[test]
fn program_heading_can_match_an_enum_type() {
    compile_and_link(
        "program cHoIcE;
         type Choice = enum Number(Value: integer); Empty; end enum;
         begin const Selected: Choice := Choice.Number(Value := 7);
           if Selected <> Choice.Number(Value := 7) then panic('enum value'); end if; end.",
        "choice",
        DefinitionTarget::Enum(0),
    );
}

#[test]
fn renamed_entry_preserves_root_bindings_and_closure_capture_provenance() {
    let linked = compile_and_link(
        "program Demo;
         function Demo(): integer; begin return 40; end function;
         begin
           const Offset: integer := Demo() - 38;
           const Calculate: function(): integer := function(): integer
             begin return Demo() + Offset; end function;
           if Calculate() <> 42 then panic('root capture'); end if;
         end.",
        "demo",
        DefinitionTarget::Function(1),
    );
    let image = linked.executable();
    let root = &image.functions[image.entry.get() as usize];
    let offset = root
        .debug
        .bindings
        .iter()
        .position(|binding| image.strings.get(binding.name) == Some("Offset"))
        .expect("source binding");
    assert!(root.debug.bindings[offset].declaration.is_some());
    assert!(!root.debug.sequence_points.is_empty());
    let closure = image
        .functions
        .iter()
        .find(|function| function.capture_count > 0)
        .expect("capturing closure");
    assert_eq!(closure.debug.lexical_owner, Some(image.entry));
    assert_eq!(
        closure.debug.capture_sources[0].binding.get() as usize,
        offset
    );
}

#[test]
fn debugger_resolves_the_source_routine_without_the_generated_entry() {
    let linked = compile_and_link(
        "program Demo;
         function Demo(): integer; begin return 42; end function;
         begin const Answer: integer := Demo();
           if Answer <> 42 then panic('routine call'); end if; end.",
        "demo",
        DefinitionTarget::Function(1),
    );
    let mut session = fpas_vm::DebugSession::new(linked).expect("debug session");
    assert_eq!(
        session.stack(0, 1).expect("root frame").items[0].name,
        "demo"
    );
    assert_eq!(
        session
            .recording_envelope()
            .expect("program identity")
            .program,
        "demo"
    );
    session.step_into().expect("stop at a source statement");
    let frame = session.stack(0, 1).expect("source frame").items[0].id;
    let result = session
        .evaluate(
            &fpas_vm::DebugExpression::Call {
                callee: Box::new(fpas_vm::DebugExpression::Callable("Demo".to_string())),
                arguments: Vec::new(),
            },
            Some(frame),
        )
        .expect("unambiguous source routine");
    assert_eq!(result.value, "42");
}
