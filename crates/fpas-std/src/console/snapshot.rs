//! Logical CRT screen snapshots for test assertions.
//!
//! **Documentation:** [`docs/pascal/std/console/README.md`](../../../../docs/pascal/std/console/README.md),
//! [`docs/pascal/std/testing/test.md`](../../../../docs/pascal/std/testing/test.md).

use super::screen::ConsoleState;

/// Captured logical screen rows from the CRT back buffer (characters only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSnapshot {
    /// Screen width in columns when the snapshot was taken.
    pub width: u16,
    /// Screen height in rows when the snapshot was taken.
    pub height: u16,
    /// One full-width row per screen line (1-based row `y` maps to index `y - 1`).
    pub rows: Vec<String>,
}

impl ScreenSnapshot {
    /// Builds a snapshot from the current console screen state.
    pub(super) fn from_state(state: &ConsoleState) -> Self {
        let rows = (1..=state.height()).map(|y| state.row_text(y)).collect();
        Self {
            width: state.width(),
            height: state.height(),
            rows,
        }
    }

    /// Returns non-empty screen content with trailing spaces and leading/trailing blank rows removed.
    pub fn compact_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .rows
            .iter()
            .map(|row| row.trim_end().to_string())
            .collect();
        while lines.first().is_some_and(|line| line.is_empty()) {
            lines.remove(0);
        }
        while lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use crate::Console;

    #[test]
    fn screen_snapshot_compact_lines_trim_blank_rows() {
        let mut console = Console::new();
        console
            .write_ln(
                &fpas_bytecode::Value::Str("Hello".into()),
                fpas_bytecode::SourceLocation::new(1, 1),
            )
            .expect("write_ln");
        let snapshot = console.screen_snapshot();
        assert_eq!(snapshot.compact_lines(), vec!["Hello".to_string()]);
    }
}
