//! Complete program-image decoding with shared debugger subtypes.

use std::hint::black_box;
use std::time::Instant;

use fpas_bytecode::{BYTECODE_VERSION, DebugType, DebugTypeId};
use fpas_program::{Digest, ProgramIdentity, ProgramImage, decode, encode};

/// Measures full image admission, excluding fixture compilation, encoding, and warmup.
pub(super) fn run(iterations: usize, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("Program artifact type depth must not exceed 64".to_owned());
    }
    let source = "program Artifact; begin end.";
    let (program, errors) = fpas_parser::parse(source);
    if !errors.is_empty() {
        return Err(format!("Artifact fixture parsing failed: {errors:?}"));
    }
    let mut executable = fpas_compiler::compile(&program)
        .map_err(|errors| format!("Artifact fixture compilation failed: {errors:?}"))?
        .into_unverified();
    executable.debug_types = vec![DebugType::Unit];
    for child in 0..depth as u32 {
        executable.debug_types.push(DebugType::Dictionary {
            key: DebugTypeId::new(child),
            value: DebugTypeId::new(child),
        });
    }
    let types = executable.debug_types.clone();
    let paths = (0..executable.source_map.sources.len())
        .map(|index| format!("source-{index}.fpas"))
        .collect::<Vec<_>>();
    let hashes = vec![Digest::of(source); paths.len()];
    let executable = executable.verify().map_err(|error| error.to_string())?;
    let image = ProgramImage::new(
        ProgramIdentity {
            compiler_version: "benchmark".to_owned(),
            bytecode_version: BYTECODE_VERSION,
            source_hash: Digest::of(source),
            options_hash: Digest::of(b"benchmark"),
            units: vec![],
        },
        paths,
        hashes,
        executable,
    )
    .map_err(|error| error.to_string())?;
    let bytes = encode(&image).map_err(|error| error.to_string())?;
    let decoded = decode(&bytes).map_err(|error| error.to_string())?;
    if decoded.executable().executable().debug_types != types {
        return Err("Program decoder changed the shared type graph".to_owned());
    }
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(decode(black_box(&bytes)).map_err(|error| error.to_string())?);
    }
    println!(
        "decodes: {iterations}\ndepth: {depth}\nelapsed: {} ms",
        started.elapsed().as_millis()
    );
    Ok(())
}
