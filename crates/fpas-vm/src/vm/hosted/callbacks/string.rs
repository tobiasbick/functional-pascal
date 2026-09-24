//! Resumable Unicode-scalar collection operations for `Std.Str`.
//!
//! **Documentation:** `docs/pascal/std/text/str/higher-order.md` (from the repository root).

use fpas_bytecode::Value;

use super::arguments::CallbackArguments;
use super::operation::{Advance, AdvanceError};

/// Input cursor and string output for `Map` and `Filter`.
pub(super) struct StringSequence {
    scalars: Vec<char>,
    next: usize,
    output: String,
}

impl StringSequence {
    /// Snapshot the input scalars before the first callback.
    pub(super) fn new(input: &str) -> Self {
        Self {
            scalars: input.chars().collect(),
            next: 0,
            output: String::with_capacity(input.len()),
        }
    }

    /// Append a mapped result only when it contains one Unicode scalar.
    pub(super) fn push_mapped(&mut self, value: Value) -> Result<(), AdvanceError> {
        match value {
            Value::Str(ref text) if text.char_len() == 1 => {
                self.output.push_str(text);
                Ok(())
            }
            other => Err(AdvanceError::InvalidStringMapResult(other)),
        }
    }

    /// Retain the scalar passed to the preceding predicate call.
    pub(super) fn push_previous(&mut self) {
        self.output.push(self.scalars[self.next - 1]);
    }

    /// Invoke the next callback or finish the string result.
    pub(super) fn next_or_string(&mut self) -> Advance {
        if let Some(scalar) = self.scalars.get(self.next).copied() {
            self.next += 1;
            Advance::Call(CallbackArguments::one(Value::Str(
                scalar.to_string().into(),
            )))
        } else {
            Advance::Complete(Value::Str(std::mem::take(&mut self.output).into()))
        }
    }
}

/// Input cursor and accumulator for `Std.Str.Reduce`.
pub(super) struct StringReduction {
    scalars: Vec<char>,
    next: usize,
    accumulator: Value,
}

impl StringReduction {
    /// Snapshot input scalars and retain the initial accumulator.
    pub(super) fn new(input: &str, initial: Value) -> Self {
        Self {
            scalars: input.chars().collect(),
            next: 0,
            accumulator: initial,
        }
    }

    /// Accept a callback result and continue left-to-right.
    pub(super) fn advance(&mut self, pending: Option<Value>) -> Advance {
        if let Some(value) = pending {
            self.accumulator = value;
        }
        if let Some(scalar) = self.scalars.get(self.next).copied() {
            self.next += 1;
            Advance::Call(CallbackArguments::two(
                self.accumulator.clone(),
                Value::Str(scalar.to_string().into()),
            ))
        } else {
            Advance::Complete(self.accumulator.clone())
        }
    }
}
