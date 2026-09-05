//! Project-aware warm, edited, and overlapping language-service queries.

mod fixture;

use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use fpas_language_service::{CancellationToken, DocumentAnalysis, LanguageService};

use fixture::Fixture;

/// Measures real project context and disk snapshots after one untimed warmup.
pub(super) fn run(queries: usize, units: usize, mode: &str) -> Result<(), String> {
    if !matches!(mode, "warm" | "edits" | "overlap") {
        return Err("Project query mode must be warm, edits, or overlap".to_owned());
    }
    let mut fixture = Fixture::create(units)?;
    let first = query(&fixture.service, &fixture.main)?;
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    if fixture.service.refresh_paths(&[], &cancellation).is_ok() {
        return Err("Cancelled project refresh unexpectedly succeeded".to_owned());
    }
    let started = Instant::now();
    for index in 0..queries {
        match mode {
            "warm" => {
                let result = query(&fixture.service, &fixture.main)?;
                if !Arc::ptr_eq(&first, &result) {
                    return Err("Unchanged project query did not reuse analysis".to_owned());
                }
                black_box(result);
            }
            "edits" => {
                let source = format!("{}\n// editor revision {index}\n", fixture.source);
                fixture
                    .service
                    .documents_mut()
                    .apply_full_text(&fixture.main, index as i64 + 2, source)
                    .map_err(|error| error.to_string())?;
                let result = query(&fixture.service, &fixture.main)?;
                if result.snapshot().revision() == first.snapshot().revision() {
                    return Err("Edited project reused the old editor revision".to_owned());
                }
                black_box(result);
            }
            "overlap" => {
                let mut left = fixture.service.fork_for_queries();
                let mut right = fixture.service.fork_for_queries();
                let main = &fixture.main;
                std::thread::scope(|scope| {
                    let left = scope.spawn(move || left.analyze_document(main));
                    let right = scope.spawn(move || right.analyze_document(main));
                    for handle in [left, right] {
                        let result = handle
                            .join()
                            .map_err(|_| "Project query thread panicked".to_owned())?
                            .map_err(|error| error.to_string())?;
                        validate(&result)?;
                        if !Arc::ptr_eq(&first, &result) {
                            return Err(
                                "Overlapping query did not reuse completed analysis".to_owned()
                            );
                        }
                        black_box(result);
                    }
                    Ok::<(), String>(())
                })?;
            }
            _ => unreachable!(),
        }
    }
    println!(
        "queries: {queries}\nunits: {units}\nmode: {mode}\nelapsed: {} ms",
        started.elapsed().as_millis()
    );
    Ok(())
}

fn query(
    service: &LanguageService,
    main: &std::path::Path,
) -> Result<Arc<DocumentAnalysis>, String> {
    let result = service
        .fork_for_queries()
        .analyze_document(main)
        .map_err(|error| error.to_string())?;
    validate(&result)?;
    Ok(result)
}

fn validate(result: &DocumentAnalysis) -> Result<(), String> {
    if result.semantic().is_none() || !result.diagnostics().is_empty() {
        return Err(format!(
            "Benchmark project must have valid semantic analysis: {:?}",
            result.diagnostics()
        ));
    }
    Ok(())
}
