//! Sparse source-run order, bounds, paths, positions, and function boundaries.

use super::{ValidationError, ValidationErrorKind};

pub(super) fn validate_source_map(executable: &crate::Executable) -> Result<(), ValidationError> {
    let mut previous = None;
    for run in &executable.source_map.runs {
        let start = run.instruction_start.get();
        if let Some(previous) = previous
            && start <= previous
        {
            return Err(ValidationError::executable(
                ValidationErrorKind::SourceRunOrder {
                    previous,
                    actual: start,
                },
            ));
        }
        let in_code = usize::try_from(start)
            .ok()
            .is_some_and(|index| index < executable.code.len());
        if !in_code {
            return Err(ValidationError::executable(
                ValidationErrorKind::SourceRunAddress {
                    actual: start,
                    code: executable.code.len(),
                },
            ));
        }
        if usize::try_from(run.source.get())
            .ok()
            .is_none_or(|index| index >= executable.source_map.sources.len())
        {
            return Err(ValidationError::executable(
                ValidationErrorKind::SourceReference {
                    actual: run.source.get(),
                    sources: executable.source_map.sources.len(),
                },
            ));
        }
        if run.line == 0 || run.column == 0 {
            return Err(ValidationError::executable(
                ValidationErrorKind::SourcePosition {
                    line: run.line,
                    column: run.column,
                },
            ));
        }
        previous = Some(start);
    }
    for function in &executable.functions {
        if executable
            .source_map
            .runs
            .binary_search_by_key(&function.code.start, |run| run.instruction_start)
            .is_err()
        {
            return Err(ValidationError::executable(
                ValidationErrorKind::MissingFunctionSource {
                    start: function.code.start.get(),
                },
            ));
        }
    }
    Ok(())
}
