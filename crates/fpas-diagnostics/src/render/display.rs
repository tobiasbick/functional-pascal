//! Whole-string Unicode widths and scalar-safe source cropping.

use unicode_width::UnicodeWidthStr;

/// Escaped source with byte boundaries for each original Unicode scalar.
pub(super) struct Line {
    text: String,
    offsets: Vec<usize>,
}

/// Expands tabs at four-cell stops while preserving indivisible escaped scalars.
pub(super) fn line(source: &str) -> Line {
    let mut text = String::new();
    let mut offsets = Vec::new();
    let mut run_start = 0;
    let mut run_column = 0;
    for character in source.chars() {
        offsets.push(text.len());
        if character == '\t' {
            // A tab ends the preceding printable run; ligatures cannot cross its spaces.
            let column = run_column + text[run_start..].width();
            let spaces = 4 - column % 4;
            text.push_str(&" ".repeat(spaces));
            run_column = column + spaces;
            run_start = text.len();
        } else {
            super::push_escaped(&mut text, &character.to_string());
        }
    }
    offsets.push(text.len());
    Line { text, offsets }
}

impl Line {
    /// Resolves a scalar coordinate, including the position after the last scalar.
    pub(super) fn offset(&self, column: u32) -> Option<usize> {
        let index = usize::try_from(column.checked_sub(1)?).ok()?;
        self.offsets.get(index).copied()
    }

    /// Returns the exclusive display-text end for a continuing multi-line range.
    pub(super) fn end(&self) -> usize {
        self.text.len()
    }
}

/// Cropped source and marker coordinates measured from the same visible substring.
pub(super) struct Window {
    pub text: String,
    pub caret: usize,
    pub length: usize,
}

/// Keeps the diagnostic start visible within the fixed 100-cell source budget.
pub(super) fn window(line: &Line, start: usize, end: usize) -> Window {
    let text = &line.text;
    let start_index = line.offsets.partition_point(|offset| *offset < start);
    let mut first = 0;
    if text.width() > 99 {
        let left = text[..start].width().saturating_sub(48);
        first = line.offsets[..=start_index]
            .partition_point(|offset| text[..*offset].width() < left)
            .min(start_index);
        // Widths may change at sequence boundaries. Always retain the selected scalar.
        if text[line.offsets[first]..start].width() > 48 {
            first = start_index;
        }
    }
    let origin = line.offsets[first];
    let prefix = if first > 0 { "..." } else { "" };
    let budget = 100 - prefix.len();
    let right = text[origin..].width() > budget;
    let limit = budget - if right { 3 } else { 0 };
    let mut last = line.offsets.len() - 1;
    if right {
        last = first
            + line.offsets[first..]
                .partition_point(|offset| text[origin..*offset].width() <= limit)
                .saturating_sub(1);
        last = last.max((start_index + 1).min(line.offsets.len() - 1));
        // Verify the actual slice rather than assuming scalar widths are additive.
        while text[origin..line.offsets[last]].width() > limit && last > start_index + 1 {
            last -= 1;
        }
    }
    let visible_end = line.offsets[last];
    let caret = prefix.len() + text[origin..start].width();
    let length = text[origin..end.min(visible_end)]
        .width()
        .saturating_sub(text[origin..start].width())
        .max(1);
    Window {
        text: format!(
            "{prefix}{}{}",
            &text[origin..visible_end],
            if last + 1 < line.offsets.len() {
                "..."
            } else {
                ""
            }
        ),
        caret,
        length,
    }
}
