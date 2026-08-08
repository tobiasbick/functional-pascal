#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "compiler integration fixtures use direct assertions for diagnostic clarity"
)]

use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_linker::link_objects;
use fpas_parser::{CompilationUnit, parse_compilation_unit};
use fpas_unit::object::{
    ChunkConstant as ObjectConstant, ChunkDefinitionKind as DefinitionKind,
    ChunkImport as ObjectImport, ChunkLocation as ObjectLocation, ChunkObject as RelocatableObject,
    collect_chunk_relocations as collect_relocations,
};

pub(crate) fn parse_unit(source: &str) -> fpas_parser::Unit {
    let (parsed, diagnostics) = parse_compilation_unit(source);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    let CompilationUnit::Unit(unit) = parsed else {
        panic!("fixture must be a unit");
    };
    unit
}

pub(crate) fn run_zero_arity(objects: Vec<RelocatableObject>, callable: &str) -> Vec<String> {
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
