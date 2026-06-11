//! Canonical formatting constants from [`style.md`](../../../docs/future/formater/style.md).

/// Spaces per indentation level.
pub const INDENT_WIDTH: usize = 2;

/// Indentation string (two spaces); equals [`INDENT_WIDTH`] spaces.
pub const INDENT: &str = "  ";

const _: () = assert!(INDENT.len() == INDENT_WIDTH);

#[cfg(test)]
mod tests {
    use super::{INDENT, INDENT_WIDTH};

    #[test]
    fn indent_matches_width() {
        assert_eq!(INDENT.len(), INDENT_WIDTH);
    }
}
