//! Bound formatting before constructing an arbitrarily large console request.

use super::*;
use std::fmt::Write;

struct Text {
    data: String,
    budget: std::sync::Arc<Budget>,
    newline: usize,
}

impl Write for Text {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let bytes = self
            .data
            .len()
            .saturating_add(text.len())
            .saturating_add(self.newline);
        if let Err(error) = self.budget.check_output(bytes) {
            heap::refuse(error);
        }
        self.data.push_str(text);
        Ok(())
    }
}

pub(crate) fn format_output(value: &crate::Value, newline: bool) -> String {
    let Some(budget) =
        current_budget().filter(|budget| budget.output_enabled.load(Ordering::Relaxed))
    else {
        return value.to_string();
    };
    let mut text = Text {
        data: String::new(),
        budget,
        newline: usize::from(newline),
    };
    let _ = write!(text, "{value}");
    text.data
}
