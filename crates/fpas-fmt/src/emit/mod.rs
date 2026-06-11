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

pub(crate) use program::{format_program, format_unit};

use crate::style::INDENT;

/// Buffered pretty-printer with indent tracking.
#[derive(Debug, Default)]
pub(crate) struct Emitter {
    out: String,
    indent_level: usize,
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

    /// Appends text without a leading indent or trailing newline.
    pub(crate) fn write(&mut self, text: &str) {
        self.out.push_str(text);
    }

    /// Appends a line with the current indent prefix.
    pub(crate) fn writeln(&mut self, line: &str) {
        self.write_indent();
        self.out.push_str(line);
        self.out.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.out.push_str(INDENT);
        }
    }

    /// Ends a statement line: `;` between statements, no extra blank line when already newline-terminated.
    pub(crate) fn finish_statement(&mut self, is_last: bool) {
        if !is_last {
            if self.out.ends_with('\n') {
                self.out.insert(self.out.len() - 1, ';');
            } else {
                self.out.push(';');
                self.out.push('\n');
            }
        } else if !self.out.ends_with('\n') {
            self.out.push('\n');
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
}
