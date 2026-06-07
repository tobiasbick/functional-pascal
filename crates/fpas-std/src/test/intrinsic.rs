//! Intrinsic dispatch for `Std.Test`.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use super::assert::{assert_equals_integer, assert_false, assert_true, fail, skip};
use crate::error::StdError;
use crate::intrinsic_args::{pop_bool, pop_int, pop_string, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, TestIntrinsic, Value};

/// Execute a `Std.Test` intrinsic; returns `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let code = match intrinsic {
        Intrinsic::Test(code) => code,
        _ => return Ok(None),
    };

    match code {
        TestIntrinsic::AssertTrue => {
            let cond = pop_bool(pop_value(stack, location)?, location)?;
            assert_true(cond, location)?;
        }
        TestIntrinsic::AssertFalse => {
            let cond = pop_bool(pop_value(stack, location)?, location)?;
            assert_false(cond, location)?;
        }
        TestIntrinsic::AssertEqualsInteger => {
            let actual = pop_int(pop_value(stack, location)?, location)?;
            let expected = pop_int(pop_value(stack, location)?, location)?;
            assert_equals_integer(expected, actual, location)?;
        }
        TestIntrinsic::Fail => {
            let msg = pop_string(pop_value(stack, location)?, location)?;
            fail(msg, location)?;
        }
        TestIntrinsic::Skip => {
            let msg = pop_string(pop_value(stack, location)?, location)?;
            skip(msg, location)?;
        }
    }

    Ok(Some(()))
}
