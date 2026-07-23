//! Stable diagnostic code catalog.
//!
//! Numeric ranges (see also [`DiagnosticCode::stage`]):
//! - **Lex:** `F0001`–`F0012`
//! - **Parse:** `F1001`–`F1999`
//! - **Sema:** `F2001`–`F2999`
//! - **Compile:** `F3001`–`F3999`
//! - **Runtime:** `F4001`–`F4999` (reserved gap: `F4016`–`F4017`)
//! - **Internal:** `F9001`–`F9999` and any other unassigned value
//!
//! Extension workflow:
//! 1. Add a new named `pub const` in the correct stage block below.
//! 2. Use the next free numeric value inside that stage range.
//! 3. Re-run `cargo test -p fpas-diagnostics` (uniqueness and stage-range tests).
//! 4. Update any diagnostic catalog docs under `docs/` if they exist.

use crate::DiagnosticCode;

// Keep each stage inventory beside its declarations so uniqueness checks stay in sync.
macro_rules! define_codes {
    ($inventory:ident => {
        $(
            $(#[$meta:meta])*
            $name:ident = $value:literal;
        )*
    }) => {
        $(
            $(#[$meta])*
            pub const $name: DiagnosticCode = DiagnosticCode::new($value);
        )*

        #[cfg(test)]
        const $inventory: &[DiagnosticCode] = &[$($name),*];
    };
}

define_codes!(LEX_ALLOCATED_CODES => {
    LEX_UNEXPECTED_CHARACTER = 1;
    LEX_UNTERMINATED_BRACE_COMMENT = 2;
    LEX_UNTERMINATED_PAREN_COMMENT = 3;
    LEX_UNTERMINATED_STRING_LITERAL = 4;
    LEX_INVALID_CHARACTER_CODE_LITERAL = 5;
    LEX_INVALID_HEXADECIMAL_LITERAL = 6;
    LEX_INTEGER_LITERAL_OVERFLOW = 7;
    LEX_REAL_LITERAL_OVERFLOW = 8;
    LEX_INVALID_NUMERIC_EXPONENT = 9;

    /// Lexer: `{$...}` is invalid source syntax.
    LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED = 10;
    /// Lexer: `__` or other invalid `_` placement inside a numeric literal.
    LEX_INVALID_DIGIT_SEPARATOR = 11;
    /// Lexer: non-ASCII letter/digit in an identifier (ASCII letters, digits, `_` only).
    LEX_NON_ASCII_IN_IDENTIFIER = 12;
});

define_codes!(PARSE_ALLOCATED_CODES => {
    PARSE_EXPECTED_TOKEN = 1001;
    PARSE_EXPECTED_IDENTIFIER = 1002;
    PARSE_INVALID_STATEMENT_START = 1003;
    PARSE_EXPECTED_TO_OR_DOWNTO = 1004;
    PARSE_EXPECTED_EXPRESSION = 1005;
    PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM = 1006;

    /// Visibility modifier (`public`/`private`) used outside a `unit` file.
    PARSE_INVALID_VISIBILITY = 1007;
    /// `static` used outside a supported static record routine.
    PARSE_INVALID_STATIC_PLACEMENT = 1008;
});

define_codes!(SEMA_ALLOCATED_CODES => {
    SEMA_UNKNOWN_TYPE = 2001;
    SEMA_DUPLICATE_DECLARATION = 2002;
    SEMA_UNKNOWN_NAME = 2003;
    SEMA_AMBIGUOUS_IMPORTED_NAME = 2004;
    SEMA_IMMUTABLE_ASSIGNMENT = 2005;
    SEMA_TYPE_MISMATCH = 2006;
    SEMA_WRONG_ARGUMENT_COUNT = 2007;
    SEMA_NON_BOOLEAN_CONDITION = 2008;
    SEMA_INVALID_PANIC_ARGUMENT = 2009;
    SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT = 2010;
    SEMA_NON_EXHAUSTIVE_CASE = 2011;
    SEMA_ENUM_FIELD_COUNT_MISMATCH = 2012;
    SEMA_CONSTRAINT_VIOLATION = 2013;
    SEMA_NON_CONSTANT_EXPRESSION = 2014;

    /// A required record field (without a default value) is missing from a record literal.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
    SEMA_MISSING_RECORD_FIELD = 2015;

    /// A task-bound callable (mutable captures) cannot cross a task boundary.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    SEMA_TASK_BOUND_CALLABLE = 2016;
});

define_codes!(COMPILE_ALLOCATED_CODES => {
    COMPILE_INVALID_DESIGNATOR_BASE = 3001;
    COMPILE_INVALID_ASSIGNMENT_TARGET = 3002;
    COMPILE_INTRINSIC_ARITY_MISMATCH = 3003;
    COMPILE_UNSUPPORTED_INTRINSIC_LOWERING_CASE = 3004;
    COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET = 3005;
    COMPILE_INVALID_GO_EXPRESSION = 3006;
    COMPILE_BYTECODE_OPERAND_OVERFLOW = 3007;
});

define_codes!(RUNTIME_ALLOCATED_CODES => {
    RUNTIME_DIVISION_BY_ZERO = 4001;
    RUNTIME_MODULO_BY_ZERO = 4002;
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS = 4003;
    RUNTIME_POP_FROM_EMPTY_ARRAY = 4004;
    RUNTIME_UNDEFINED_GLOBAL = 4005;
    RUNTIME_UNDEFINED_FUNCTION = 4006;
    RUNTIME_WRONG_CALL_ARITY = 4007;

    /// Operand has the wrong dynamic type for the operation (including std intrinsic argument checks).
    RUNTIME_VM_OPERAND_TYPE_MISMATCH = 4008;

    /// Intrinsic stack underflow, or an argument violates an intrinsic precondition (not a dynamic type mismatch).
    RUNTIME_INTRINSIC_STACK_STATE_ERROR = 4009;
    RUNTIME_PROGRAM_PANIC = 4010;
    RUNTIME_CONSOLE_INPUT_FAILURE = 4011;
    RUNTIME_NUMERIC_DOMAIN_ERROR = 4012;
    RUNTIME_CONVERSION_FAILURE = 4013;
    RUNTIME_CONSOLE_STATE_ERROR = 4014;
    RUNTIME_UNWRAP_FAILURE = 4015;
    // Reserved: 4016–4017 (gap before task/runtime codes; do not reuse without audit).
    RUNTIME_INVALID_TASK = 4018;
    RUNTIME_DICT_KEY_NOT_FOUND = 4019;
    RUNTIME_VM_SHUTDOWN = 4020;
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS = 4021;

    /// `Std.Str.Format`: specifier count does not match argument list, or a type does not match its specifier.
    RUNTIME_FORMAT_MISMATCH = 4022;

    /// `Std.Test` assertion or explicit `Fail` call.
    RUNTIME_TEST_ASSERTION_FAILED = 4023;
});

define_codes!(INTERNAL_ALLOCATED_CODES => {
    INTERNAL_COMPILER_INVARIANT_FAILURE = 9001;
    INTERNAL_VM_INVARIANT_FAILURE = 9002;
});

#[cfg(test)]
const ALL_CODE_INVENTORIES: &[&[DiagnosticCode]] = &[
    LEX_ALLOCATED_CODES,
    PARSE_ALLOCATED_CODES,
    SEMA_ALLOCATED_CODES,
    COMPILE_ALLOCATED_CODES,
    RUNTIME_ALLOCATED_CODES,
    INTERNAL_ALLOCATED_CODES,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiagnosticStage;
    use std::collections::HashSet;

    #[test]
    fn allocated_codes_are_unique() {
        let mut seen = HashSet::new();
        for stage_codes in ALL_CODE_INVENTORIES {
            for code in stage_codes.iter().copied() {
                assert!(
                    seen.insert(code.value()),
                    "duplicate diagnostic code allocation detected: {code}",
                );
            }
        }
    }

    #[test]
    fn allocated_codes_match_stage_ranges() {
        for code in LEX_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Lex,
                "lex catalog code {code} is outside the lex range"
            );
        }
        for code in PARSE_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Parse,
                "parse catalog code {code} is outside the parse range"
            );
        }
        for code in SEMA_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Sema,
                "sema catalog code {code} is outside the sema range"
            );
        }
        for code in COMPILE_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Compile,
                "compile catalog code {code} is outside the compile range"
            );
        }
        for code in RUNTIME_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Runtime,
                "runtime catalog code {code} is outside the runtime range"
            );
        }
        for code in INTERNAL_ALLOCATED_CODES {
            assert_eq!(
                code.stage(),
                DiagnosticStage::Internal,
                "internal catalog code {code} is outside the internal range"
            );
        }
    }
}
