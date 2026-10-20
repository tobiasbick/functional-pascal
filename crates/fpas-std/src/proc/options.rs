//! Pure construction of process options without native record field defaults.

use crate::{AggregateFactory, StdError, std_symbols};
use fpas_bytecode::{SourceLocation, Value};

pub(super) fn defaults(
    factory: &dyn AggregateFactory,
    location: SourceLocation,
) -> Result<Value, StdError> {
    factory.record(
        std_symbols::STD_PROC_PROCESS_OPTIONS,
        vec![
            Value::OptionNone,
            Value::Array(Vec::new().into()),
            Value::Str("".into()),
            Value::OptionNone,
            Value::Integer(1_048_576),
            Value::OptionNone,
        ],
        location,
    )
}
