//! Recovery for comment forms from Pascal (`{…}`, `(*…*)`) and Ada (`--`).
//!
//! **Documentation:** `docs/pascal/language/basics/comments.md`

use crate::lexer::Lexer;
use fpas_diagnostics::codes::LEX_INVALID_COMMENT_FORM;
use fpas_diagnostics::{Applicability, Fix, fix_title};

const COMMENT_HINT: &str = "Use `// comment`. For multiple lines, prefix each line with `//`.";

impl Lexer<'_> {
    /// Consumes `{...}` or the remaining source and reports one actionable diagnostic.
    pub(in crate::lexer) fn scan_invalid_brace_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        let mut closed = false;
        while !self.at_end() {
            closed = self.current() == b'}';
            self.advance();
            if closed {
                break;
            }
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`{...}` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
        self.attach_comment_fix(so, 1, closed);
    }

    /// Consumes `(*...*)` or the remaining source and reports one actionable diagnostic.
    pub(in crate::lexer) fn scan_invalid_paren_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        let mut closed = false;
        while !self.at_end() {
            if self.current() == b'*' && self.peek_at(1) == Some(b')') {
                self.advance();
                self.advance();
                closed = true;
                break;
            }
            self.advance();
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`(*...*)` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
        self.attach_comment_fix(so, 2, closed);
    }
}

impl Lexer<'_> {
    /// Adds a `//` rewrite to the comment diagnostic just pushed for the closed comment from
    /// `start` to the current position, whose delimiters are `delimiter` bytes wide.
    ///
    /// Only comments that end their line get a fix, so no code after them becomes commented
    /// out. A comment over several lines must also start its line; each line gets `//`.
    fn attach_comment_fix(&mut self, start: usize, delimiter: usize, closed: bool) {
        if !closed {
            return;
        }
        let rest_of_line = self.source[self.pos..]
            .split(['\n', '\r'])
            .next()
            .unwrap_or_default();
        if !rest_of_line.trim().is_empty() {
            return;
        }
        let inner = &self.source[start + delimiter..self.pos - delimiter];
        let line_start = self.source[..start]
            .rfind(['\n', '\r'])
            .map_or(0, |index| index + 1);
        let indent = &self.source[line_start..start];
        let lines: Vec<&str> = inner.lines().map(str::trim).collect();
        let new_text = if lines.len() <= 1 {
            line_comment(lines.first().copied().unwrap_or_default())
        } else if indent.trim().is_empty() {
            let newline = if inner.contains("\r\n") { "\r\n" } else { "\n" };
            let first = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
            let last = lines.iter().rposition(|line| !line.is_empty()).unwrap_or(0);
            lines[first..=last.max(first)]
                .iter()
                .map(|line| line_comment(line))
                .collect::<Vec<_>>()
                .join(&format!("{newline}{indent}"))
        } else {
            return;
        };
        let Some(span) = self
            .errors
            .last()
            .and_then(|diagnostic| diagnostic.span)
            .filter(|span| span.offset() == start)
        else {
            return;
        };
        let fix = Fix::replace(
            fix_title("// comment"),
            Applicability::MachineApplicable,
            span,
            new_text,
        );
        if let Some(diagnostic) = self.errors.pop() {
            self.errors.push(diagnostic.with_fix(fix));
        }
    }
}

fn line_comment(text: &str) -> String {
    if text.is_empty() {
        "//".to_owned()
    } else {
        format!("// {text}")
    }
}

impl Lexer<'_> {
    /// True at an Ada `--` comment: at the start of a line or after a statement terminator, so
    /// `X - -1` stays an expression.
    pub(in crate::lexer) fn at_ada_comment(&self) -> bool {
        if self.current() != b'-' || self.peek_at(1) != Some(b'-') {
            return false;
        }
        let before = self.source.get(..self.pos).unwrap_or_default();
        let line_start = before
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .all(|ch| ch == ' ' || ch == '\t' || ch == '\r');
        line_start
            || self
                .tokens
                .last()
                .is_some_and(|token| token.token == crate::Token::Semicolon)
    }

    /// Consumes an Ada `--` comment through the end of its line and reports it.
    pub(in crate::lexer) fn scan_ada_comment(&mut self) {
        let start = self.span_here();
        while !self.at_end() && !matches!(self.current(), b'\n' | b'\r') {
            self.advance_utf8_char();
        }
        let text = self
            .source
            .get(start.0 + 2..self.pos)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let replacement = format!("// {text}");
        self.report_foreign_token(
            "Ada `--` comment is not FPAS syntax",
            &replacement,
            "Comments start with `//`.",
            start,
            Some(Applicability::MachineApplicable),
        );
    }
}
