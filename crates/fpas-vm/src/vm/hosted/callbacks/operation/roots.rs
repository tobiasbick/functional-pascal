//! Every retained callback cursor, partial result and accumulator is an execution edge.

use super::*;
use fpas_bytecode::{OwnershipError, TraceRoots};

impl TraceRoots for CallbackContinuation {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        visit(&Value::Function(self.callback.clone()));
        if let Some(value) = &self.pending {
            visit(value);
        }
        self.operation.trace_roots(visit)
    }
}

impl TraceRoots for CallbackOperation {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        match self {
            Self::ArrayMap(sequence)
            | Self::ArrayFilter(sequence)
            | Self::ArrayFlatMap(sequence) => {
                sequence.cursor.values.as_slice().trace_roots(visit)?;
                sequence.output.as_slice().trace_roots(visit)?;
            }
            Self::ArraySortBy(sort) => sort.trace_roots(visit)?,
            Self::ArrayReduce {
                values,
                accumulator,
                ..
            } => {
                values.as_slice().trace_roots(visit)?;
                visit(accumulator);
            }
            Self::ArrayFind(cursor)
            | Self::ArrayFindIndex(cursor)
            | Self::ArrayAny(cursor)
            | Self::ArrayAll(cursor)
            | Self::ArrayForEach(cursor) => cursor.values.as_slice().trace_roots(visit)?,
            Self::DictMap(dictionary) | Self::DictFilter(dictionary) => {
                pairs(&dictionary.entries, visit);
                pairs(&dictionary.output, visit);
            }
            Self::DictReduce(reduction) => {
                pairs(&reduction.entries, visit);
                visit(&reduction.accumulator);
            }
            Self::StringReduce(reduction) => reduction.trace_roots(visit)?,
            Self::Single {
                arguments: Some(arguments),
                ..
            }
            | Self::Mcp {
                arguments: Some(arguments),
                ..
            } => arguments.as_slice().trace_roots(visit)?,
            Self::Single {
                arguments: None, ..
            }
            | Self::Mcp {
                arguments: None, ..
            }
            | Self::StringMap(_)
            | Self::StringFilter(_) => {}
        }
        Ok(())
    }
}

fn pairs(values: &[(Value, Value)], visit: &mut dyn FnMut(&Value)) {
    for (key, value) in values {
        visit(key);
        visit(value);
    }
}

#[cfg(test)]
mod tests;
