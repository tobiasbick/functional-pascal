//! Direct lengths and successful unwraps bypass general Std dispatch on unary calls.
//!
//! Semantics: `docs/pascal/language/types/array/README.md`,
//! `docs/pascal/language/types/string/README.md` and
//! `docs/pascal/language/types/dictionary-operations.md`,
//! `docs/pascal/language/types/option-operations.md` and
//! `docs/pascal/language/types/result-operations.md`.

use crate::{
    AbcOperands, ArrayIntrinsic, DictIntrinsic, Intrinsic, NO_REGISTER, OptionIntrinsic,
    ResultIntrinsic, StrIntrinsic, Value,
};

use crate::execution::vm::{VmError, worker::Worker};

const ARRAY_LENGTH: u16 = ArrayIntrinsic::Length as u16;
const STRING_LENGTH: u16 = StrIntrinsic::Length as u16;
const DICTIONARY_LENGTH: u16 = DictIntrinsic::Length as u16;
const OPTION_UNWRAP: u16 = OptionIntrinsic::Unwrap as u16;
const RESULT_UNWRAP: u16 = ResultIntrinsic::Unwrap as u16;

impl Worker {
    #[inline(always)]
    pub(super) fn execute_unary_intrinsic(
        &mut self,
        operands: AbcOperands,
    ) -> Result<bool, VmError> {
        let intrinsic = match operands.b {
            ARRAY_LENGTH => Intrinsic::Array(ArrayIntrinsic::Length),
            STRING_LENGTH => Intrinsic::Str(StrIntrinsic::Length),
            DICTIONARY_LENGTH => Intrinsic::Dict(DictIntrinsic::Length),
            OPTION_UNWRAP => Intrinsic::Option(OptionIntrinsic::Unwrap),
            RESULT_UNWRAP => Intrinsic::Result(ResultIntrinsic::Unwrap),
            _ => return Ok(false),
        };
        self.check_scheduling_effect(intrinsic)?;
        let Some(argument) = self
            .base
            .checked_add(usize::from(operands.c))
            .and_then(|index| self.registers.get(index))
        else {
            return Ok(false);
        };
        let length = match (intrinsic, argument) {
            (Intrinsic::Array(_), Value::Array(values)) => values.len(),
            (Intrinsic::Str(_), Value::Str(text)) => text.char_len(),
            (Intrinsic::Dict(_), Value::Dict(entries)) => entries.len(),
            (Intrinsic::Option(_), Value::OptionSome(inner))
            | (Intrinsic::Result(_), Value::ResultOk(inner)) => {
                if operands.a != NO_REGISTER {
                    let value = (**inner).clone();
                    self.write_operand(operands.a, value)?;
                }
                return Ok(true);
            }
            // The shared implementation owns failure diagnostics and source locations.
            _ => return Ok(false),
        };
        if operands.a != NO_REGISTER {
            self.write_integer_operand(operands.a, length as i64)?;
        }
        Ok(true)
    }
}
