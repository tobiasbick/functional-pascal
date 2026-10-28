//! Allocation budgets shared by all values in an inspection capture.

/// Remaining charged storage and value slots; callers may share it across frames.
pub(in crate::execution::vm) struct ValueBudget {
    /// Number of additional values that may be retained.
    pub values: usize,
    /// Remaining charged bytes, including node and child-vector storage.
    pub bytes: usize,
    /// Most recent allocation limit in the current value capture.
    pub exhausted: Option<&'static str>,
}

impl ValueBudget {
    /// A bounded independent inspection request.
    pub(in crate::execution::vm) fn new(values: usize, bytes: usize) -> Self {
        Self {
            values,
            bytes,
            exhausted: None,
        }
    }

    /// Charge bytes before allocating retained data.
    pub(in crate::execution::vm) fn charge(&mut self, bytes: usize) -> bool {
        if bytes > self.bytes {
            self.exhausted = Some("retained_byte_limit");
            return false;
        }
        self.bytes -= bytes;
        true
    }

    /// Reserve a node before reading or rendering its contents.
    pub(in crate::execution::vm) fn node(&mut self, bytes: usize) -> bool {
        if self.values == 0 {
            self.exhausted = Some("value_limit");
            return false;
        }
        if !self.charge(bytes) {
            return false;
        }
        self.values -= 1;
        true
    }

    /// Copy a Unicode prefix that fits both the string and retained-byte limits.
    pub(in crate::execution::vm) fn text(&mut self, text: &str, maximum: usize) -> (String, bool) {
        let mut end = 0;
        for (index, character) in text.char_indices().take(maximum) {
            let next = index + character.len_utf8();
            if next > self.bytes {
                self.exhausted = Some("retained_byte_limit");
                break;
            }
            end = next;
        }
        self.bytes -= end;
        (text[..end].to_string(), end < text.len())
    }
}
