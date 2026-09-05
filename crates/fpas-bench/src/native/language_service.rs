//! Repeated semantic queries against one unchanged editor buffer.

use std::fmt::Write;
use std::hint::black_box;
use std::time::Instant;

use fpas_language_service::{LanguageService, WorkspaceContext};

/// Measures fresh query services after parsing and a warmup analysis.
pub(super) fn run(queries: usize, functions: usize) -> Result<(), String> {
    let root = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join(".temp-data/bench/native-analysis");
    // Keep discovery inside this fixture rather than scanning changing sibling benchmarks.
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let path = root.join("query.fpas");
    let mut source = String::from("program QueryBenchmark;\n");
    for index in 0..functions {
        writeln!(
            source,
            "function F{index}(X: integer): integer; begin return X + {index} end;"
        )
        .map_err(|error| error.to_string())?;
    }
    source.push_str("begin var Value: integer := F0(1) end.\n");
    let mut service = LanguageService::new(WorkspaceContext::loose(&root));
    service
        .documents_mut()
        .open_document(&path, 1, source)
        .map_err(|error| error.to_string())?;
    let warmup = service
        .fork_for_queries()
        .analyze_document(&path)
        .map_err(|error| error.to_string())?;
    if warmup.semantic().is_none() || !warmup.diagnostics().is_empty() {
        return Err("Native analysis benchmark must have valid semantic analysis".to_owned());
    }
    let started = Instant::now();
    for _ in 0..queries {
        black_box(
            service
                .fork_for_queries()
                .analyze_document(black_box(&path))
                .map_err(|error| error.to_string())?,
        );
    }
    println!(
        "queries: {queries}\nfunctions: {functions}\nelapsed: {} ms",
        started.elapsed().as_millis()
    );
    Ok(())
}
