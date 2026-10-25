//! Production error rendering, essential image metadata and constructor sharing.
use std::time::Instant;

use fpas_bytecode::{BYTECODE_VERSION, DebugType, Value, VerifiedExecutable};
use fpas_program::{Digest, ProgramIdentity, ProgramImage, decode, encode};
use fpas_vm::Vm;

const CAUSES: &str = include_str!("../../../../examples/pascal/errors/cause_benchmark.fpas");
const CONSTRUCTION: &str =
    include_str!("../../../../examples/pascal/errors/construction_benchmark.fpas");

pub(super) fn run(iterations: usize, mode: &str) -> Result<(), String> {
    if iterations == 0 || iterations > 10_000 {
        return Err("Error workload iterations must be within 1..10000".into());
    }
    match mode {
        "message" | "console" => render(iterations, mode),
        "deep" => deep(iterations),
        "metadata" => metadata(),
        _ => Err("Error workload mode must be message, console, deep or metadata".into()),
    }
}

fn compile(source: &str) -> Result<VerifiedExecutable, String> {
    let (program, diagnostics) = fpas_parser::parse(source);
    if !diagnostics.is_empty() {
        return Err(format!("Error fixture parse diagnostics: {diagnostics:?}"));
    }
    fpas_compiler::compile(&program).map_err(|errors| format!("Error fixture: {errors:?}"))
}

fn render(iterations: usize, mode: &str) -> Result<(), String> {
    let executable = compile(CAUSES)?;
    let mut vm = Vm::with_args(executable, vec![mode.into(), iterations.to_string()]);
    vm.run().map_err(|error| error.message.clone())?;
    let output = vm.output();
    let rendered = if mode == "console" { iterations } else { 0 };
    let expected = "ErrorCauses.Chain.Link: Context\ncaused by ".repeat(16)
        + "ErrorCauses.Chain.Leaf: Terminal\n  Code = 7";
    if output.lines.len() != rendered + 2
        || output.lines[..rendered]
            .iter()
            .any(|line| line != &expected)
        || output.lines[rendered] != format!("checksum: {}", iterations * expected.len())
    {
        return Err("Captured error output differs from the independent golden".into());
    }
    // The source timer surrounds rendering/capture only; compilation, admission,
    // cause construction, warmup and this complete output check are outside it.
    println!("boundary: {mode} (host-free, depth 16)");
    println!("{}\n{}", output.lines[rendered], output.lines[rendered + 1]);
    Ok(())
}

fn image(executable: VerifiedExecutable, source: &str) -> Result<ProgramImage, String> {
    let paths = (0..executable.executable().source_map.sources.len())
        .map(|index| format!("error-source-{index}.fpas"))
        .collect::<Vec<_>>();
    let hashes = vec![Digest::of(source); paths.len()];
    ProgramImage::new(
        ProgramIdentity {
            contracts_enabled: true,
            compiler_version: "error-benchmark".into(),
            bytecode_version: BYTECODE_VERSION,
            source_hash: Digest::of(source),
            options_hash: Digest::of(b"errors"),
            units: vec![],
        },
        paths,
        hashes,
        executable,
    )
    .map_err(|error| error.to_string())
}

fn deep(iterations: usize) -> Result<(), String> {
    let mut executable = compile(CAUSES)?.into_unverified();
    // Cause must survive without optional enum debugger types.
    for ty in &mut executable.debug_types {
        if matches!(ty, DebugType::Enum(_)) {
            *ty = DebugType::Dynamic;
        }
    }
    let executable = executable.verify().map_err(|error| error.to_string())?;
    let bytes = encode(&image(executable, CAUSES)?).map_err(|error| error.to_string())?;
    let check = || -> Result<(), String> {
        let decoded = decode(&bytes).map_err(|error| error.to_string())?;
        let mut vm = Vm::with_args(
            decoded.executable().clone(),
            vec!["deep".into(), "1".into()],
        );
        vm.run().map_err(|error| error.message.clone())?;
        let expected = 512 * "ErrorCauses.Chain.Link: Context\ncaused by ".len()
            + "ErrorCauses.Chain.Leaf: Terminal\n  Code = 7".len();
        if vm.output().lines.first() != Some(&format!("checksum: {expected}")) {
            return Err("Deep decoded Cause failed structural/output validation".into());
        }
        Ok(())
    };
    check()?;
    let started = Instant::now();
    for _ in 0..iterations {
        check()?;
    }
    println!(
        "depth: 512\nroundtrips: {iterations}\nimage_bytes: {}\nelapsed: {} ms",
        bytes.len(),
        started.elapsed().as_millis()
    );
    Ok(())
}

fn metadata() -> Result<(), String> {
    let executable = compile(CONSTRUCTION)?;
    let data = executable.executable();
    let instances = data
        .enums
        .iter()
        .filter_map(|owner| owner.error.as_ref())
        .filter(|key| key.declaration == "errorconstruction.wrapped")
        .collect::<Vec<_>>();
    if instances.len() != 2 || instances[0] == instances[1] {
        return Err("Generic error instances were duplicated or erased".into());
    }
    let factories = data
        .functions
        .iter()
        .filter(|function| data.strings.get(function.name) == Some("Wrapped.Item"))
        .collect::<Vec<_>>();
    if factories.len() != 2 {
        return Err(format!(
            "Expected two shared generic factories, got {}; functions: {:?}",
            factories.len(),
            data.functions
                .iter()
                .filter_map(|function| data.strings.get(function.name))
                .collect::<Vec<_>>()
        ));
    }
    let instructions: u32 = factories.iter().filter_map(|f| f.code.len()).sum();
    println!(
        "value_bytes: {}\nconcrete_instances: 2\nshared_factories: 2\nfactory_instruction_bytes: {}",
        std::mem::size_of::<Value>(),
        instructions as usize * std::mem::size_of::<fpas_bytecode::Instruction>()
    );
    let keys = data
        .enums
        .iter()
        .filter_map(|owner| owner.error.as_ref())
        .collect::<Vec<_>>();
    let variants = data
        .enum_variants
        .iter()
        .filter_map(|variant| {
            variant.error.as_ref().map(|error| {
                serde_json::json!({
                    "fields": error.field_types,
                    "cause": error.cause.map(|cause| (cause.field, cause.family.get())),
                    "message": error.message.and_then(|id| data.strings.get(id))
                })
            })
        })
        .collect::<Vec<_>>();
    let metadata_bytes = serde_json::to_vec(&(keys, variants))
        .map_err(|error| error.to_string())?
        .len();
    println!("error_metadata_json_bytes: {metadata_bytes}");
    let bytes = encode(&image(executable, CONSTRUCTION)?).map_err(|e| e.to_string())?;
    println!("construction_image_bytes: {}", bytes.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn actual_error_workloads_validate_rendering_and_stripped_image_causes() {
        super::run(1, "message").expect("Message renderer");
        super::run(1, "console").expect("captured Console renderer");
        super::run(1, "deep").expect("deep essential image metadata");
        super::run(1, "metadata").expect("shared generic constructor factories");
    }
}
