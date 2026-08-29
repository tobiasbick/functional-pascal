//! Intrinsic dispatch for `Std.Test`.
//!
//! **Documentation:** `docs/pascal/std/testing/test.md` (from the repository root).

use super::assert::{
    assert_equals_boolean, assert_equals_integer, assert_equals_real, assert_equals_string,
    assert_false, assert_true, fail, skip,
};
use crate::error::{StdError, std_internal_error};
use crate::intrinsic_args::{IntrinsicCall, pop_bool, pop_int, pop_real, pop_string, pop_value};
use fpas_bytecode::{Intrinsic, SourceLocation, TestIntrinsic};

/// Execute a `Std.Test` intrinsic; returns `None` when another unit should handle it.
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    let code = match intrinsic {
        Intrinsic::Test(code) => code,
        _ => return Ok(None),
    };

    match code {
        TestIntrinsic::AssertTrue => {
            let cond = pop_bool(pop_value(call, location)?, location)?;
            assert_true(cond, location)?;
        }
        TestIntrinsic::AssertFalse => {
            let cond = pop_bool(pop_value(call, location)?, location)?;
            assert_false(cond, location)?;
        }
        TestIntrinsic::AssertEqualsInteger => {
            let actual = pop_int(pop_value(call, location)?, location)?;
            let expected = pop_int(pop_value(call, location)?, location)?;
            assert_equals_integer(expected, actual, location)?;
        }
        TestIntrinsic::AssertEqualsBoolean => {
            let actual = pop_bool(pop_value(call, location)?, location)?;
            let expected = pop_bool(pop_value(call, location)?, location)?;
            assert_equals_boolean(expected, actual, location)?;
        }
        TestIntrinsic::AssertEqualsString => {
            let actual = pop_string(pop_value(call, location)?, location)?;
            let expected = pop_string(pop_value(call, location)?, location)?;
            assert_equals_string(expected, actual, location)?;
        }
        TestIntrinsic::AssertEqualsReal => {
            let actual = pop_real(pop_value(call, location)?, location)?;
            let expected = pop_real(pop_value(call, location)?, location)?;
            assert_equals_real(expected, actual, location)?;
        }
        TestIntrinsic::Fail => {
            let msg = pop_string(pop_value(call, location)?, location)?;
            fail(msg, location)?;
        }
        TestIntrinsic::Skip => {
            let msg = pop_string(pop_value(call, location)?, location)?;
            skip(msg, location)?;
        }
        TestIntrinsic::AssertScreenLine
        | TestIntrinsic::AssertScreenCell
        | TestIntrinsic::PushReadLn
        | TestIntrinsic::ScratchDir => {
            return Err(std_internal_error(
                "internal: Std.Test input/screen/view assertions are handled in the VM",
                "This indicates a VM dispatch bug. Please report this as a compiler/runtime bug.",
                location,
            ));
        }
    }

    Ok(Some(()))
}
