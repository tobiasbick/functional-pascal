//! Decode and validate an object with repeatedly shared debugger subtypes.

use std::hint::black_box;
use std::time::Instant;

use fpas_unit::object::{OBJECT_VERSION, ObjectDebugType, RelocatableObject, decode_object};

/// Measures the production decoder on a valid shared type graph, excluding fixture setup.
pub(super) fn run(iterations: usize, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("Artifact type depth must not exceed 64".to_owned());
    }
    let mut types = vec![ObjectDebugType::Integer];
    for child in 0..depth as u32 {
        types.push(ObjectDebugType::Dictionary {
            key: child,
            value: child,
        });
    }
    let object = RelocatableObject {
        version: OBJECT_VERSION,
        owner: "benchmark".to_owned(),
        entry: None,
        initializer: None,
        functions: vec![],
        constants: vec![],
        globals: vec![],
        records: vec![],
        enums: vec![],
        debug_types: types,
        sources: vec![],
        definitions: vec![],
        imports: vec![],
        relocations: vec![],
    };
    let bytes = serde_json::to_vec(&object).map_err(|error| error.to_string())?;
    let decoded = decode_object(&bytes).map_err(|error| error.to_string())?;
    if decoded != object {
        return Err("Artifact decoder changed the shared type graph".to_owned());
    }
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(decode_object(black_box(&bytes)).map_err(|error| error.to_string())?);
    }
    println!(
        "decodes: {iterations}\ndepth: {depth}\nelapsed: {} ms",
        started.elapsed().as_millis()
    );
    Ok(())
}
