use std::collections::BTreeMap;

use fpas_bytecode::Op;
use fpas_unit::object::{
    ChunkConstant as ObjectConstant, ChunkLocation as ObjectLocation,
    ChunkObject as RelocatableObject, collect_chunk_relocations as collect_relocations,
};

pub fn object(owner: &str, code: Vec<Op>, constants: Vec<ObjectConstant>) -> RelocatableObject {
    RelocatableObject {
        owner: owner.to_string(),
        locations: vec![
            ObjectLocation {
                line: 1,
                column: 1,
                source_id: 0,
            };
            code.len()
        ],
        relocations: collect_relocations(&code),
        code,
        constants,
        functions: BTreeMap::new(),
        definitions: Vec::new(),
        imports: Vec::new(),
    }
}
