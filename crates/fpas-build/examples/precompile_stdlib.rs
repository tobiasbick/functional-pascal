//! Distribution-staging helper that compiles source-adjacent standard-library sidecars.

use std::path::PathBuf;

fn main() -> Result<(), String> {
    let source_root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(usage)?;
    let destination_root = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or_else(usage)?;
    let counters = fpas_build::stage_standard_library(
        &source_root,
        &destination_root,
        &fpas_build::BuildOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    eprintln!(
        "Compiled {} standard-library unit(s), reused {}.",
        counters.compiled, counters.sidecar_reused
    );
    Ok(())
}

fn usage() -> String {
    "usage: precompile_stdlib <staging-library-directory> <distribution-library-directory>"
        .to_string()
}
