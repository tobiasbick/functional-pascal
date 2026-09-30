//! Compiler-local source labels and their translation back to graph source paths.
//!
//! Objects carry the label `source-{id}.fpas`, where `id` indexes the unit graph's source table.
//! The linker renumbers sources in first-use order, so a linked executable's `SourceId` is not a
//! graph index; diagnostics must translate through the executable's source labels.

use std::path::PathBuf;

use fpas_bytecode::VerifiedExecutable;

/// Return the compiler-local label of one unit-graph source.
pub(crate) fn source_label(source_id: u32) -> String {
    format!("source-{source_id}.fpas")
}

fn graph_index(label: &str) -> Option<usize> {
    label
        .strip_prefix("source-")?
        .strip_suffix(".fpas")?
        .parse()
        .ok()
}

/// Return diagnostic source paths indexed by the linked executable's `SourceId`.
///
/// Labels produced by this crate resolve through `graph_sources`; any other entry, such as a
/// portable path already installed by a program image, is used as written.
#[must_use]
pub fn linked_source_paths(
    executable: &VerifiedExecutable,
    graph_sources: &[PathBuf],
) -> Vec<PathBuf> {
    let executable = executable.executable();
    executable
        .source_map
        .sources
        .iter()
        .map(|id| {
            let label = executable.strings.get(*id).unwrap_or_default();
            graph_index(label)
                .and_then(|index| graph_sources.get(index).cloned())
                .unwrap_or_else(|| PathBuf::from(label))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{graph_index, source_label};

    #[test]
    fn labels_round_trip_to_graph_indexes() {
        assert_eq!(graph_index(&source_label(0)), Some(0));
        assert_eq!(graph_index(&source_label(37)), Some(37));
        assert_eq!(graph_index("units/std/console.fpas"), None);
        assert_eq!(graph_index("source-x.fpas"), None);
    }
}
