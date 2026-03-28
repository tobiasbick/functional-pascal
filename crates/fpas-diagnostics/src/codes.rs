//! Stable diagnostic code catalog.
//!
//! Extension workflow:
//! 1. Add a new named `pub const` in the correct stage range.
//! 2. Keep constants grouped by stage and use the next free numeric value
//!    inside that range.
//! 3. Add the new constant to `allocated_codes_are_unique` below.
//!    The test rejects accidental code reuse across the full catalog.
//! 4. Re-run `cargo test --workspace` and update `docs/plans/diagnostics/02-code-catalog.md`.

use crate::DiagnosticCode;

pub const LEX_UNEXPECTED_CHARACTER: DiagnosticCode = DiagnosticCode::new(1);
pub const LEX_UNTERMINATED_BRACE_COMMENT: DiagnosticCode = DiagnosticCode::new(2);
pub const LEX_UNTERMINATED_PAREN_COMMENT: DiagnosticCode = DiagnosticCode::new(3);
pub const LEX_UNTERMINATED_STRING_LITERAL: DiagnosticCode = DiagnosticCode::new(4);
pub const LEX_INVALID_CHARACTER_CODE_LITERAL: DiagnosticCode = DiagnosticCode::new(5);
pub const LEX_INVALID_HEXADECIMAL_LITERAL: DiagnosticCode = DiagnosticCode::new(6);
pub const LEX_INTEGER_LITERAL_OVERFLOW: DiagnosticCode = DiagnosticCode::new(7);
pub const LEX_REAL_LITERAL_OVERFLOW: DiagnosticCode = DiagnosticCode::new(8);
pub const LEX_INVALID_NUMERIC_EXPONENT: DiagnosticCode = DiagnosticCode::new(9);

pub const PARSE_EXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::new(1001);
pub const PARSE_EXPECTED_IDENTIFIER: DiagnosticCode = DiagnosticCode::new(1002);
pub const PARSE_INVALID_STATEMENT_START: DiagnosticCode = DiagnosticCode::new(1003);
pub const PARSE_EXPECTED_TO_OR_DOWNTO: DiagnosticCode = DiagnosticCode::new(1004);
pub const PARSE_EXPECTED_EXPRESSION: DiagnosticCode = DiagnosticCode::new(1005);
pub const PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM: DiagnosticCode = DiagnosticCode::new(1006);

pub const SEMA_UNKNOWN_TYPE: DiagnosticCode = DiagnosticCode::new(2001);
pub const SEMA_DUPLICATE_DECLARATION: DiagnosticCode = DiagnosticCode::new(2002);
pub const SEMA_UNKNOWN_NAME: DiagnosticCode = DiagnosticCode::new(2003);
pub const SEMA_AMBIGUOUS_IMPORTED_NAME: DiagnosticCode = DiagnosticCode::new(2004);
pub const SEMA_IMMUTABLE_ASSIGNMENT: DiagnosticCode = DiagnosticCode::new(2005);
pub const SEMA_TYPE_MISMATCH: DiagnosticCode = DiagnosticCode::new(2006);
pub const SEMA_WRONG_ARGUMENT_COUNT: DiagnosticCode = DiagnosticCode::new(2007);
pub const SEMA_NON_BOOLEAN_CONDITION: DiagnosticCode = DiagnosticCode::new(2008);
pub const SEMA_INVALID_PANIC_ARGUMENT: DiagnosticCode = DiagnosticCode::new(2009);
pub const SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT: DiagnosticCode = DiagnosticCode::new(2010);
pub const SEMA_NON_EXHAUSTIVE_CASE: DiagnosticCode = DiagnosticCode::new(2011);
pub const SEMA_ENUM_FIELD_COUNT_MISMATCH: DiagnosticCode = DiagnosticCode::new(2012);

pub const COMPILE_INVALID_DESIGNATOR_BASE: DiagnosticCode = DiagnosticCode::new(3001);
pub const COMPILE_INVALID_ASSIGNMENT_TARGET: DiagnosticCode = DiagnosticCode::new(3002);
pub const COMPILE_INTRINSIC_ARITY_MISMATCH: DiagnosticCode = DiagnosticCode::new(3003);
pub const COMPILE_UNSUPPORTED_INTRINSIC_LOWERING_CASE: DiagnosticCode = DiagnosticCode::new(3004);
pub const COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET: DiagnosticCode = DiagnosticCode::new(3005);
pub const COMPILE_INVALID_GO_EXPRESSION: DiagnosticCode = DiagnosticCode::new(3006);

pub const RUNTIME_DIVISION_BY_ZERO: DiagnosticCode = DiagnosticCode::new(4001);
pub const RUNTIME_MODULO_BY_ZERO: DiagnosticCode = DiagnosticCode::new(4002);
pub const RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS: DiagnosticCode = DiagnosticCode::new(4003);
pub const RUNTIME_POP_FROM_EMPTY_ARRAY: DiagnosticCode = DiagnosticCode::new(4004);
pub const RUNTIME_UNDEFINED_GLOBAL: DiagnosticCode = DiagnosticCode::new(4005);
pub const RUNTIME_UNDEFINED_FUNCTION: DiagnosticCode = DiagnosticCode::new(4006);
pub const RUNTIME_WRONG_CALL_ARITY: DiagnosticCode = DiagnosticCode::new(4007);
pub const RUNTIME_VM_OPERAND_TYPE_MISMATCH: DiagnosticCode = DiagnosticCode::new(4008);
pub const RUNTIME_INTRINSIC_STACK_STATE_ERROR: DiagnosticCode = DiagnosticCode::new(4009);
pub const RUNTIME_PROGRAM_PANIC: DiagnosticCode = DiagnosticCode::new(4010);
pub const RUNTIME_CONSOLE_INPUT_FAILURE: DiagnosticCode = DiagnosticCode::new(4011);
pub const RUNTIME_NUMERIC_DOMAIN_ERROR: DiagnosticCode = DiagnosticCode::new(4012);
pub const RUNTIME_CONVERSION_FAILURE: DiagnosticCode = DiagnosticCode::new(4013);
pub const RUNTIME_CONSOLE_STATE_ERROR: DiagnosticCode = DiagnosticCode::new(4014);
pub const RUNTIME_UNWRAP_FAILURE: DiagnosticCode = DiagnosticCode::new(4015);
pub const RUNTIME_CHANNEL_CLOSED: DiagnosticCode = DiagnosticCode::new(4016);
pub const RUNTIME_INVALID_CHANNEL: DiagnosticCode = DiagnosticCode::new(4017);
pub const RUNTIME_INVALID_TASK: DiagnosticCode = DiagnosticCode::new(4018);
pub const RUNTIME_DICT_KEY_NOT_FOUND: DiagnosticCode = DiagnosticCode::new(4019);
pub const RUNTIME_VM_SHUTDOWN: DiagnosticCode = DiagnosticCode::new(4020);

pub const INTERNAL_COMPILER_INVARIANT_FAILURE: DiagnosticCode = DiagnosticCode::new(9001);
pub const INTERNAL_VM_INVARIANT_FAILURE: DiagnosticCode = DiagnosticCode::new(9002);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn allocated_codes_are_unique() {
        let allocated_codes = [
            LEX_UNEXPECTED_CHARACTER,
            LEX_UNTERMINATED_BRACE_COMMENT,
            LEX_UNTERMINATED_PAREN_COMMENT,
            LEX_UNTERMINATED_STRING_LITERAL,
            LEX_INVALID_CHARACTER_CODE_LITERAL,
            LEX_INVALID_HEXADECIMAL_LITERAL,
            LEX_INTEGER_LITERAL_OVERFLOW,
            LEX_REAL_LITERAL_OVERFLOW,
            LEX_INVALID_NUMERIC_EXPONENT,
            PARSE_EXPECTED_TOKEN,
            PARSE_EXPECTED_IDENTIFIER,
            PARSE_INVALID_STATEMENT_START,
            PARSE_EXPECTED_TO_OR_DOWNTO,
            PARSE_EXPECTED_EXPRESSION,
            PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM,
            SEMA_UNKNOWN_TYPE,
            SEMA_DUPLICATE_DECLARATION,
            SEMA_UNKNOWN_NAME,
            SEMA_AMBIGUOUS_IMPORTED_NAME,
            SEMA_IMMUTABLE_ASSIGNMENT,
            SEMA_TYPE_MISMATCH,
            SEMA_WRONG_ARGUMENT_COUNT,
            SEMA_NON_BOOLEAN_CONDITION,
            SEMA_INVALID_PANIC_ARGUMENT,
            SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT,
            SEMA_NON_EXHAUSTIVE_CASE,
            SEMA_ENUM_FIELD_COUNT_MISMATCH,
            COMPILE_INVALID_DESIGNATOR_BASE,
            COMPILE_INVALID_ASSIGNMENT_TARGET,
            COMPILE_INTRINSIC_ARITY_MISMATCH,
            COMPILE_UNSUPPORTED_INTRINSIC_LOWERING_CASE,
            COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET,
            COMPILE_INVALID_GO_EXPRESSION,
            RUNTIME_DIVISION_BY_ZERO,
            RUNTIME_MODULO_BY_ZERO,
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            RUNTIME_POP_FROM_EMPTY_ARRAY,
            RUNTIME_UNDEFINED_GLOBAL,
            RUNTIME_UNDEFINED_FUNCTION,
            RUNTIME_WRONG_CALL_ARITY,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            RUNTIME_PROGRAM_PANIC,
            RUNTIME_CONSOLE_INPUT_FAILURE,
            RUNTIME_NUMERIC_DOMAIN_ERROR,
            RUNTIME_CONVERSION_FAILURE,
            RUNTIME_CONSOLE_STATE_ERROR,
            RUNTIME_UNWRAP_FAILURE,
            RUNTIME_CHANNEL_CLOSED,
            RUNTIME_INVALID_CHANNEL,
            RUNTIME_INVALID_TASK,
            RUNTIME_DICT_KEY_NOT_FOUND,
            RUNTIME_VM_SHUTDOWN,
            INTERNAL_COMPILER_INVARIANT_FAILURE,
            INTERNAL_VM_INVARIANT_FAILURE,
        ];

        let mut seen = HashSet::new();
        for code in allocated_codes {
            assert!(
                seen.insert(code.value()),
                "duplicate diagnostic code allocation detected: {code}",
            );
        }
    }
}
