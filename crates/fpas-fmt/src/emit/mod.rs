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
mod width;

pub(crate) use program::{format_program, format_unit};

use crate::style::{INDENT, MAX_LINE_WIDTH};

/// Buffered pretty-printer with indent tracking.
#[derive(Debug, Default)]
pub(crate) struct Emitter {
    out: String,
    indent_level: usize,
    layout: std::sync::Arc<crate::layout::SourceLayout>,
    continuation_indent: Option<usize>,
    measuring: bool,
    suffix_width: usize,
}

impl Emitter {
    /// Creates an empty emitter.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns a byte checkpoint for detecting line breaks in subsequent emission.
    pub(crate) fn output_len(&self) -> usize {
        self.out.len()
    }

    /// Returns whether output after a byte checkpoint contains a line break.
    pub(crate) fn has_newline_since(&self, offset: usize) -> bool {
        self.out
            .get(offset..)
            .is_some_and(|text| text.contains('\n'))
    }

    /// Starts a content line at the current indentation, reusing an empty active line.
    pub(crate) fn start_line(&mut self) {
        while matches!(self.out.as_bytes().last(), Some(b' ' | b'\t')) {
            self.out.pop();
        }
        if !self.out.is_empty() && !self.ends_with_newline() {
            self.write_line_end();
        }
        self.write_current_indent();
    }

    /// Creates a compact measurement emitter without automatic wrapping.
    pub(crate) fn for_measurement() -> Self {
        Self {
            measuring: true,
            ..Self::default()
        }
    }

    /// Measures source-preserved layout without introducing automatic breaks.
    pub(crate) fn for_source_measurement(comments: &crate::comments::CommentMap) -> Self {
        Self {
            measuring: true,
            layout: comments.layout.clone(),
            ..Self::default()
        }
    }

    /// Returns whether adding measured text and its suffix would exceed the target width.
    pub(crate) fn would_exceed_width(&self, width: Option<usize>, suffix: usize) -> bool {
        !self.measuring
            && width.is_some_and(|width| {
                let current = self.out.rsplit('\n').next().unwrap_or_default();
                width::text_width(current) + width + suffix + self.suffix_width > MAX_LINE_WIDTH
            })
    }

    /// Reserves width for following postfix operations, restoring the enclosing reservation.
    pub(crate) fn with_suffix_width<R>(
        &mut self,
        width: usize,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = self.suffix_width;
        self.suffix_width = width;
        let result = f(self);
        self.suffix_width = previous;
        result
    }

    /// Creates an emitter with the original source layout.
    pub(crate) fn with_comments(comments: &crate::comments::CommentMap) -> Self {
        Self {
            layout: comments.layout.clone(),
            ..Self::default()
        }
    }

    /// Runs expression emission at a structural indentation, then restores its context.
    pub(crate) fn at_indent<R>(&mut self, spaces: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_level = self.indent_level;
        let old_continuation = self.continuation_indent;
        self.indent_level = spaces / INDENT.len();
        self.continuation_indent = Some(spaces);
        let result = f(self);
        self.indent_level = old_level;
        self.continuation_indent = old_continuation;
        result
    }

    /// Returns the accumulated output without trailing horizontal whitespace.
    #[must_use]
    pub(crate) fn finish(self) -> String {
        let mut normalized = String::with_capacity(self.out.len());
        for ch in self.out.chars() {
            if ch == '\n' {
                while matches!(normalized.as_bytes().last(), Some(b' ' | b'\t')) {
                    normalized.pop();
                }
            }
            normalized.push(ch);
        }
        while matches!(normalized.as_bytes().last(), Some(b' ' | b'\t')) {
            normalized.pop();
        }
        normalized
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

    /// Leading-space width of the active line, i.e. the indentation it started with.
    pub(crate) fn line_indent(&self) -> usize {
        let line = self.out.rsplit('\n').next().unwrap_or_default();
        line.len() - line.trim_start_matches(' ').len()
    }

    /// Appends a newline followed by the requested structural indentation.
    pub(crate) fn newline_to_indent(&mut self, spaces: usize) {
        self.out.push('\n');
        for _ in 0..spaces {
            self.out.push(' ');
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

    /// Appends the current indent prefix without advancing to a new line.
    pub(crate) fn write_current_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.out.push_str(INDENT);
        }
    }

    fn write_indent(&mut self) {
        self.write_current_indent();
    }

    /// Returns `true` when the buffer ends with a newline.
    pub(crate) fn ends_with_newline(&self) -> bool {
        self.out.ends_with('\n')
    }

    /// Appends a newline.
    pub(crate) fn write_line_end(&mut self) {
        self.out.push('\n');
    }

    /// Removes one trailing newline so a separator or end-of-line comment can precede it.
    pub(crate) fn remove_line_end(&mut self) {
        debug_assert!(self.out.ends_with('\n'));
        self.out.pop();
    }

    /// Ends a single-line statement with its required `;` and a newline.
    pub(crate) fn finish_line_statement(&mut self) {
        self.out.push(';');
        self.out.push('\n');
    }

    /// Ends a multi-line statement that already ends with a newline (e.g. `end` on its own line).
    ///
    /// Inserts `;` immediately before that trailing newline.
    pub(crate) fn finish_statement_after_newline(&mut self) {
        debug_assert!(
            self.out.ends_with('\n'),
            "finish_statement_after_newline requires output to end with a newline"
        );
        self.out.insert(self.out.len() - 1, ';');
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
        emitter.with_indent(|e| e.writeln("WriteLn('hi');"));
        assert_eq!(emitter.finish(), "program Hello;\n\n  WriteLn('hi');\n");
    }

    #[test]
    fn finish_removes_trailing_horizontal_whitespace() {
        let mut emitter = Emitter::new();
        emitter.write("value  ");
        emitter.write_line_end();
        emitter.with_indent(|inner| {
            inner.write_current_indent();
            inner.write_line_end();
        });
        emitter.write("tail\t");

        assert_eq!(emitter.finish(), "value\n\ntail");
    }

    #[test]
    fn finish_line_statement_adds_semicolon_and_newline() {
        let mut emitter = Emitter::new();
        emitter.write("WriteLn('ok')");
        emitter.finish_line_statement();
        assert_eq!(emitter.finish(), "WriteLn('ok');\n");
    }

    #[test]
    fn finish_statement_after_newline_inserts_before_trailing_newline() {
        let mut emitter = Emitter::new();
        emitter.writeln("end");
        emitter.finish_statement_after_newline();
        assert_eq!(emitter.finish(), "end;\n");
    }
}
