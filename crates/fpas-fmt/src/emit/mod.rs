//! AST-to-text emission.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "emit API used from stmt/decl/program and unit tests"
    )
)]

mod decl;
mod expr;
mod program;
mod stmt;
mod types;
mod wrap;

pub(crate) use program::{format_program, format_unit};

use crate::style::INDENT;

/// Buffered pretty-printer with indent tracking.
#[derive(Debug, Default)]
pub(crate) struct Emitter {
    out: String,
    indent_level: usize,
    column: usize,
}

impl Emitter {
    /// Creates an empty emitter.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns the accumulated output.
    #[must_use]
    pub(crate) fn finish(self) -> String {
        self.out
    }

    /// Current indent depth in levels (each level is two spaces).
    pub(crate) fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Increases indent depth by one.
    pub(crate) fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decreases indent depth by one (saturating at zero).
    pub(crate) fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    /// Runs `f` with one extra indent level, then restores the previous level.
    pub(crate) fn with_indent<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.indent();
        let result = f(self);
        self.dedent();
        result
    }

    /// Appends one blank line when the buffer is non-empty and does not already end with `\n\n`.
    pub(crate) fn blank_line(&mut self) {
        if self.out.is_empty() {
            return;
        }
        if self.out.ends_with("\n\n") {
            return;
        }
        if self.out.ends_with('\n') {
            self.out.push('\n');
        } else {
            self.out.push_str("\n\n");
        }
    }

    /// Current column on the active line (includes leading spaces).
    pub(crate) fn column(&self) -> usize {
        self.column
    }

    /// Appends a newline and spaces to reach `target_column`, resetting the active column.
    pub(crate) fn newline_to_column(&mut self, target_column: usize) {
        self.out.push('\n');
        for _ in 0..target_column {
            self.out.push(' ');
        }
        self.column = target_column;
    }

    /// Appends text without a leading indent or trailing newline.
    pub(crate) fn write(&mut self, text: &str) {
        for ch in text.chars() {
            self.out.push(ch);
            if ch == '\n' {
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
    }

    /// Appends a line with the current indent prefix.
    pub(crate) fn writeln(&mut self, line: &str) {
        self.write_indent();
        self.out.push_str(line);
        self.out.push('\n');
        self.column = 0;
    }

    /// Appends the current indent prefix without advancing to a new line.
    pub(crate) fn write_current_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.out.push_str(INDENT);
        }
        self.column = self.indent_level * INDENT.len();
    }

    fn write_indent(&mut self) {
        self.write_current_indent();
    }

    /// Returns `true` when the buffer ends with a newline.
    pub(crate) fn ends_with_newline(&self) -> bool {
        self.out.ends_with('\n')
    }

    /// Appends a newline and resets the active column.
    pub(crate) fn write_line_end(&mut self) {
        self.out.push('\n');
        self.column = 0;
    }

    /// Ends a single-line statement: appends `;` and a newline when not last in the block.
    pub(crate) fn finish_line_statement(&mut self, is_last: bool) {
        if !is_last {
            self.out.push(';');
            self.out.push('\n');
            self.column = 0;
        } else if !self.out.ends_with('\n') {
            self.out.push('\n');
            self.column = 0;
        }
    }

    /// Ends a multi-line statement that already ends with a newline (e.g. `end` on its own line).
    ///
    /// When not last in the block, inserts `;` immediately before that trailing newline.
    pub(crate) fn finish_statement_after_newline(&mut self, is_last: bool) {
        debug_assert!(
            self.out.ends_with('\n'),
            "finish_statement_after_newline requires output to end with a newline"
        );
        if !is_last {
            self.out.insert(self.out.len() - 1, ';');
            self.column = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Emitter;

    #[test]
    fn emitter_indent_and_blank_line() {
        let mut emitter = Emitter::new();
        emitter.writeln("program Hello;");
        emitter.blank_line();
        emitter.with_indent(|e| e.writeln("WriteLn('hi')"));
        assert_eq!(emitter.finish(), "program Hello;\n\n  WriteLn('hi')\n");
    }

    #[test]
    fn finish_line_statement_adds_semicolon_and_newline() {
        let mut emitter = Emitter::new();
        emitter.write("WriteLn('ok')");
        emitter.finish_line_statement(false);
        assert_eq!(emitter.finish(), "WriteLn('ok');\n");
    }

    #[test]
    fn finish_line_statement_last_omits_semicolon() {
        let mut emitter = Emitter::new();
        emitter.write("WriteLn('ok')");
        emitter.finish_line_statement(true);
        assert_eq!(emitter.finish(), "WriteLn('ok')\n");
    }

    #[test]
    fn finish_statement_after_newline_inserts_before_trailing_newline() {
        let mut emitter = Emitter::new();
        emitter.writeln("end");
        emitter.finish_statement_after_newline(false);
        assert_eq!(emitter.finish(), "end;\n");
    }

    #[test]
    fn finish_statement_after_newline_last_keeps_trailing_newline_only() {
        let mut emitter = Emitter::new();
        emitter.writeln("end");
        emitter.finish_statement_after_newline(true);
        assert_eq!(emitter.finish(), "end\n");
    }
}
