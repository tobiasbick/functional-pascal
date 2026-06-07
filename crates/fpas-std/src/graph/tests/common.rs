//! Shared helpers for graph session tests.

use fpas_bytecode::SourceLocation;

use super::super::with_headless_graph_backend_for_tests;

pub(super) fn test_location() -> SourceLocation {
    SourceLocation::new(1, 1)
}

pub(super) fn with_headless<T>(f: impl FnOnce() -> T) -> T {
    with_headless_graph_backend_for_tests(f)
}
