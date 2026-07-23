//! Distribution-staging helper that compiles source-adjacent standard-library sidecars.

use std::path::PathBuf;

fn main() -> Result<(), String> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: precompile_stdlib <standard-library-directory>".to_string())?;
    let library = fpas_project::load_standard_library(&root)?;
    let graph = fpas_project::build_unit_graph_with_standard_library(
        &[],
        &fpas_project::ProjectLinkMeta::default(),
        &library,
    )?;
    let selection = fpas_project::resolve_library_units(&graph)?;
    let built =
        fpas_build::build_library_units(&graph, &selection, &fpas_build::BuildOptions::default())
            .map_err(|error| error.to_string())?;
    let counters = built.counters();
    eprintln!(
        "Prepared {} standard-library unit(s), reused {}.",
        counters.compiled, counters.sidecar_reused
    );
    Ok(())
}
