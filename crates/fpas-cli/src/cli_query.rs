//! Shared schema, bounded preparation and output for source queries and refactoring.

mod projection;
mod protocol;
#[path = "cli_refactor/mod.rs"]
mod refactor;
mod report;
mod request;
mod worker;
pub(crate) use request::{cwd_failure, run, selected};
pub(crate) use worker::worker;

fn source_offset(
    snapshot: &fpas_language_service::DocumentSnapshot,
    line: usize,
    column: usize,
) -> Option<usize> {
    let range = snapshot.line_index().line_range(line.checked_sub(1)?)?;
    let text = snapshot.source().get(range.clone())?;
    let index = column.checked_sub(1)?;
    let byte_column = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(text.len()))
        .nth(index)?;
    Some(range.start + byte_column)
}
