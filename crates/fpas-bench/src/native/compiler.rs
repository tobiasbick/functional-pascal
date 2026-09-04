//! Semantic analysis and lowering of one function with many sequential branches.

use std::hint::black_box;
use std::time::Instant;

/// Measures lowering after parsing and a warmup compilation, without VM execution.
pub(super) fn run(iterations: usize, branches: usize) -> Result<(), String> {
    let mut source =
        String::from("program BranchBenchmark; begin mutable var Value: integer := 0;\n");
    for _ in 0..branches {
        source.push_str("if Value mod 2 = 0 then Value := Value + 1 else Value := Value + 2;\n");
    }
    source.push_str("end.\n");
    let (program, errors) = fpas_parser::parse(&source);
    if !errors.is_empty() {
        return Err(format!("Compiler benchmark parsing failed: {errors:?}"));
    }
    black_box(
        fpas_compiler::lower(&program).map_err(|errors| format!("Lowering failed: {errors:?}"))?,
    );
    let started = Instant::now();
    for _ in 0..iterations {
        black_box(
            fpas_compiler::lower(black_box(&program))
                .map_err(|errors| format!("Lowering failed: {errors:?}"))?,
        );
    }
    println!(
        "iterations: {iterations}\nbranches: {branches}\nelapsed: {} ms",
        started.elapsed().as_millis()
    );
    Ok(())
}
