//! Canonical formatting constants from [`fmt-style.md`](../../../docs/pascal/tools/fmt-style.md).

/// Target display width for automatic expression wrapping.
pub const MAX_LINE_WIDTH: usize = 100;

/// Spaces per indentation level.
pub const INDENT_WIDTH: usize = 2;

/// Indentation string (two spaces); equals [`INDENT_WIDTH`] spaces.
pub const INDENT: &str = "  ";

const _: () = assert!(INDENT.len() == INDENT_WIDTH);
