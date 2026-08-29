//! State machine for one resumable higher-order operation.

use fpas_bytecode::{SharedFunction, Value};

/// Complete resumable state for one higher-order intrinsic owned by a task.
pub(in crate::vm) struct CallbackContinuation {
    pub(super) callback: SharedFunction,
    pub(super) destination: Option<usize>,
    pub(super) operation: CallbackOperation,
    pending: Option<Value>,
    pub(super) awaiting_depth: Option<usize>,
}

/// Partial result and cursor state for each supported callback intrinsic.
pub(super) enum CallbackOperation {
    ArrayMap(Sequence),
    ArrayFilter(Sequence),
    ArrayReduce {
        values: Vec<Value>,
        next: usize,
        accumulator: Value,
    },
    ArrayFind(Cursor),
    ArrayFindIndex(Cursor),
    ArrayAny(Cursor),
    ArrayAll(Cursor),
    ArrayFlatMap(Sequence),
    ArrayForEach(Cursor),
    DictMap(Dictionary),
    DictFilter(Dictionary),
    Single {
        arguments: Option<Vec<Value>>,
        wrapper: SingleWrapper,
    },
}

/// Cursor over immutable callback inputs.
pub(super) struct Cursor {
    pub(super) values: Vec<Value>,
    next: usize,
}

/// Cursor and accumulated array output.
pub(super) struct Sequence {
    cursor: Cursor,
    output: Vec<Value>,
}

/// Cursor and accumulated dictionary output.
pub(super) struct Dictionary {
    pub(super) entries: Vec<(Value, Value)>,
    next: usize,
    output: Vec<(Value, Value)>,
}

#[derive(Clone, Copy)]
/// Result wrapper applied after a single callback invocation.
pub(super) enum SingleWrapper {
    Direct,
    OptionSome,
    ResultOk,
}

/// Next action requested by the hosted-operation state machine.
pub(super) enum Advance {
    Call(Vec<Value>),
    Complete(Value),
}

impl CallbackContinuation {
    /// Create a continuation before its first callback invocation.
    pub(super) fn new(
        callback: SharedFunction,
        destination: Option<usize>,
        operation: CallbackOperation,
    ) -> Self {
        Self {
            callback,
            destination,
            operation,
            pending: None,
            awaiting_depth: None,
        }
    }

    /// Return whether a call stack at this depth is returning the hosted callback.
    pub(super) fn awaits_depth(&self, depth: usize) -> bool {
        self.awaiting_depth == Some(depth)
    }

    /// Retain a completed callback value for the next operation step.
    pub(super) fn accept(&mut self, value: Value) {
        debug_assert!(self.awaiting_depth.take().is_some());
        self.pending = Some(value);
    }

    /// Consume a pending result and choose the next invocation or final value.
    pub(super) fn advance(&mut self) -> Result<Advance, Value> {
        match &mut self.operation {
            CallbackOperation::ArrayMap(sequence) => {
                sequence.push_pending(self.pending.take());
                Ok(sequence.next_or_array())
            }
            CallbackOperation::ArrayFilter(sequence) => {
                if let Some(value) = self.pending.take()
                    && boolean(value)?
                {
                    sequence.push_previous();
                }
                Ok(sequence.next_or_array())
            }
            CallbackOperation::ArrayReduce {
                values,
                next,
                accumulator,
            } => {
                if let Some(value) = self.pending.take() {
                    *accumulator = value;
                }
                if let Some(value) = values.get(*next).cloned() {
                    *next += 1;
                    Ok(Advance::Call(vec![accumulator.clone(), value]))
                } else {
                    Ok(Advance::Complete(accumulator.clone()))
                }
            }
            CallbackOperation::ArrayFind(cursor) => {
                if let Some(value) = self.pending.take()
                    && boolean(value)?
                {
                    return Ok(Advance::Complete(Value::OptionSome(Box::new(
                        cursor.previous().clone(),
                    ))));
                }
                Ok(cursor
                    .next()
                    .map_or(Advance::Complete(Value::OptionNone), |value| {
                        Advance::Call(vec![value])
                    }))
            }
            CallbackOperation::ArrayFindIndex(cursor) => {
                if let Some(value) = self.pending.take()
                    && boolean(value)?
                {
                    return Ok(Advance::Complete(Value::Integer((cursor.next - 1) as i64)));
                }
                Ok(cursor
                    .next()
                    .map_or(Advance::Complete(Value::Integer(-1)), |value| {
                        Advance::Call(vec![value])
                    }))
            }
            CallbackOperation::ArrayAny(cursor) => {
                if let Some(value) = self.pending.take()
                    && boolean(value)?
                {
                    return Ok(Advance::Complete(Value::Boolean(true)));
                }
                Ok(cursor
                    .next()
                    .map_or(Advance::Complete(Value::Boolean(false)), |value| {
                        Advance::Call(vec![value])
                    }))
            }
            CallbackOperation::ArrayAll(cursor) => {
                if let Some(value) = self.pending.take()
                    && !boolean(value)?
                {
                    return Ok(Advance::Complete(Value::Boolean(false)));
                }
                Ok(cursor
                    .next()
                    .map_or(Advance::Complete(Value::Boolean(true)), |value| {
                        Advance::Call(vec![value])
                    }))
            }
            CallbackOperation::ArrayFlatMap(sequence) => {
                if let Some(value) = self.pending.take() {
                    match value {
                        Value::Array(inner) => sequence.output.extend(inner.iter().cloned()),
                        other => sequence.output.push(other),
                    }
                }
                Ok(sequence.next_or_array())
            }
            CallbackOperation::ArrayForEach(cursor) => {
                self.pending.take();
                Ok(cursor
                    .next()
                    .map_or(Advance::Complete(Value::Unit), |value| {
                        Advance::Call(vec![value])
                    }))
            }
            CallbackOperation::DictMap(dictionary) => {
                if let Some(value) = self.pending.take() {
                    dictionary.push_previous_value(value);
                }
                Ok(dictionary.next_map())
            }
            CallbackOperation::DictFilter(dictionary) => {
                if let Some(value) = self.pending.take()
                    && boolean(value)?
                {
                    dictionary.push_previous_entry();
                }
                Ok(dictionary.next_filter())
            }
            CallbackOperation::Single { arguments, wrapper } => {
                if let Some(value) = self.pending.take() {
                    return Ok(Advance::Complete(wrapper.wrap(value)));
                }
                Ok(Advance::Call(
                    arguments.take().expect("single callback runs once"),
                ))
            }
        }
    }
}

impl CallbackOperation {
    /// Return the visible arity of the first callback invocation.
    pub(super) fn first_arity(&self) -> usize {
        match self {
            Self::ArrayReduce { .. } | Self::DictFilter(_) => 2,
            Self::Single { arguments, .. } => arguments.as_ref().map_or(0, Vec::len),
            _ => 1,
        }
    }
}

impl Cursor {
    /// Create a cursor before the first value.
    pub(super) fn new(values: Vec<Value>) -> Self {
        Self { values, next: 0 }
    }

    fn next(&mut self) -> Option<Value> {
        let value = self.values.get(self.next)?.clone();
        self.next += 1;
        Some(value)
    }

    fn previous(&self) -> &Value {
        &self.values[self.next - 1]
    }
}

impl Sequence {
    /// Create an empty array result for the supplied inputs.
    pub(super) fn new(values: Vec<Value>) -> Self {
        Self {
            cursor: Cursor::new(values),
            output: Vec::new(),
        }
    }

    fn push_pending(&mut self, value: Option<Value>) {
        self.output.extend(value);
    }

    fn push_previous(&mut self) {
        self.output.push(self.cursor.previous().clone());
    }

    fn next_or_array(&mut self) -> Advance {
        self.cursor.next().map_or_else(
            || Advance::Complete(Value::Array(std::mem::take(&mut self.output).into())),
            |value| Advance::Call(vec![value]),
        )
    }
}

impl Dictionary {
    /// Create an empty dictionary result for the supplied entries.
    pub(super) fn new(entries: Vec<(Value, Value)>) -> Self {
        Self {
            entries,
            next: 0,
            output: Vec::new(),
        }
    }

    fn push_previous_value(&mut self, value: Value) {
        self.output
            .push((self.entries[self.next - 1].0.clone(), value));
    }

    fn push_previous_entry(&mut self) {
        self.output.push(self.entries[self.next - 1].clone());
    }

    fn next_map(&mut self) -> Advance {
        if let Some((_, value)) = self.entries.get(self.next) {
            self.next += 1;
            Advance::Call(vec![value.clone()])
        } else {
            Advance::Complete(Value::dict(std::mem::take(&mut self.output)))
        }
    }

    fn next_filter(&mut self) -> Advance {
        if let Some((key, value)) = self.entries.get(self.next) {
            self.next += 1;
            Advance::Call(vec![key.clone(), value.clone()])
        } else {
            Advance::Complete(Value::dict(std::mem::take(&mut self.output)))
        }
    }
}

impl SingleWrapper {
    fn wrap(self, value: Value) -> Value {
        match self {
            Self::Direct => value,
            Self::OptionSome => Value::OptionSome(Box::new(value)),
            Self::ResultOk => Value::ResultOk(Box::new(value)),
        }
    }
}

fn boolean(value: Value) -> Result<bool, Value> {
    match value {
        Value::Boolean(value) => Ok(value),
        other => Err(other),
    }
}
