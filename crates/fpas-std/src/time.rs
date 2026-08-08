//! `Std.Time` runtime implementation.
//!
//! Wall-clock and monotonic time helpers plus blocking sleep.
//!
//! **Documentation:** `docs/pascal/std/host/time.md` (from the repository root).

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_int, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, TimeIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static MONOTONIC_ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Execute a `Std.Time` intrinsic and return `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Time(TimeIntrinsic::TimestampMillis) => {
            call.push(Value::Integer(timestamp_millis()));
        }
        Intrinsic::Time(TimeIntrinsic::MonotonicMillis) => {
            call.push(Value::Integer(monotonic_millis()));
        }
        Intrinsic::Time(TimeIntrinsic::ElapsedMillis) => {
            let start = pop_int(pop_value(call, location)?, location)?;
            call.push(Value::Integer(elapsed_millis(start)));
        }
        Intrinsic::Time(TimeIntrinsic::Sleep) => {
            let milliseconds = pop_int(pop_value(call, location)?, location)?;
            sleep_millis(milliseconds, location)?;
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

fn timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as i64
}

fn monotonic_millis() -> i64 {
    MONOTONIC_ORIGIN.elapsed().as_millis() as i64
}

fn elapsed_millis(start: i64) -> i64 {
    (monotonic_millis() - start).max(0)
}

fn sleep_millis(milliseconds: i64, location: SourceLocation) -> Result<(), StdError> {
    if milliseconds < 0 {
        return Err(std_runtime_error(
            RUNTIME_NUMERIC_DOMAIN_ERROR,
            format!("Sleep expects a non-negative millisecond count, got {milliseconds}"),
            "Pass `0` or a positive integer number of milliseconds.",
            location,
        ));
    }
    thread::sleep(Duration::from_millis(milliseconds as u64));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_location() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn run_time(intrinsic: TimeIntrinsic, stack: &mut Vec<Value>) {
        crate::run_intrinsic(Intrinsic::Time(intrinsic), stack, test_location()).unwrap();
    }

    #[test]
    fn timestamp_millis_returns_positive_epoch_value() {
        let mut stack = Vec::new();
        run_time(TimeIntrinsic::TimestampMillis, &mut stack);
        let Value::Integer(timestamp) = stack[0] else {
            panic!("expected integer timestamp");
        };
        assert!(timestamp > 1_000_000_000_000);
    }

    #[test]
    fn monotonic_millis_is_non_negative() {
        let mut stack = Vec::new();
        run_time(TimeIntrinsic::MonotonicMillis, &mut stack);
        assert_eq!(stack, vec![Value::Integer(monotonic_millis())]);
    }

    #[test]
    fn elapsed_millis_measures_sleep_duration() {
        let start = monotonic_millis();
        sleep_millis(20, test_location()).unwrap();
        let mut stack = vec![Value::Integer(start)];
        run_time(TimeIntrinsic::ElapsedMillis, &mut stack);
        let Value::Integer(elapsed) = stack[0] else {
            panic!("expected elapsed integer");
        };
        assert!(elapsed >= 15);
    }

    #[test]
    fn sleep_rejects_negative_milliseconds() {
        let err = sleep_millis(-1, test_location()).expect_err("negative sleep");
        assert!(err.message.contains("non-negative"));
    }
}
