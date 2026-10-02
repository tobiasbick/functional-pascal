//! Stable diagnostic code catalog.
//!
//! Numeric ranges (see also [`DiagnosticCode::stage`]):
//! - **Lex:** `F0001`–`F0013`
//! - **Parse:** `F1001`–`F1999`
//! - **Sema:** `F2001`–`F2999`
//! - **Compile:** `F3001`–`F3999`
//! - **Runtime:** `F4001`–`F4999` (reserved gap: `F4017`)
//! - **Project/build:** `F5001`–`F5999`
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
            #[doc = concat!("Stable diagnostic code `", stringify!($name), "`.")]
            pub const $name: DiagnosticCode = DiagnosticCode::new($value);
        )*

        #[doc = concat!("Stable allocated diagnostic inventory `", stringify!($inventory), "`.")]
        pub const $inventory: &[DiagnosticCode] = &[$($name),*];
    };
}

define_codes!(LEX_ALLOCATED_CODES => {
    LEX_UNEXPECTED_CHARACTER = 1;
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
    /// Lexer: a comment-like form other than `//` was used.
    LEX_INVALID_COMMENT_FORM = 13;
});

define_codes!(PARSE_ALLOCATED_CODES => {
    PARSE_EXPECTED_TOKEN = 1001;
    PARSE_EXPECTED_IDENTIFIER = 1002;
    PARSE_INVALID_STATEMENT_START = 1003;
    PARSE_EXPECTED_TO_OR_DOWNTO = 1004;
    PARSE_EXPECTED_EXPRESSION = 1005;
    PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM = 1006;

    /// The `public` visibility modifier was used outside a `unit` file.
    PARSE_INVALID_VISIBILITY = 1007;
    /// `static` used outside a supported static record routine.
    PARSE_INVALID_STATIC_PLACEMENT = 1008;
    /// Parser recursion exceeded the compiler's shared nesting budget.
    ///
    /// **Documentation:** `docs/pascal/program-structure/cli.md` (Checking without running)
    PARSE_NESTING_LIMIT_EXCEEDED = 1009;
    /// A record update expression contains no field assignments.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
    PARSE_EMPTY_RECORD_UPDATE = 1010;
    /// An event declaration places its `write` accessor before its `read` accessor.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    PARSE_INVALID_EVENT_ACCESSOR_ORDER = 1011;
    /// An enum variant declares an empty associated-data field list.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    PARSE_EMPTY_ENUM_FIELD_LIST = 1012;
    /// An enum variant field list ends with a trailing semicolon.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    PARSE_TRAILING_ENUM_FIELD_SEPARATOR = 1013;
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

    /// A non-public record member is accessed outside its declaring unit.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md`
    SEMA_PRIVATE_RECORD_MEMBER = 2017;

    /// An implicit enum backing value would exceed the signed 64-bit integer range.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    SEMA_ENUM_BACKING_VALUE_EXHAUSTED = 2018;

    /// A public unit declaration refers to a type that is private to that unit.
    ///
    /// **Documentation:** `docs/pascal/program-structure/visibility.md`
    SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE = 2019;
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

    /// A retained task was cancelled by the debugger without running its remaining body.
    RUNTIME_TASK_CANCELLED = 4016;
    // Reserved: 4017 (gap before task/runtime codes; do not reuse without audit).
    RUNTIME_INVALID_TASK = 4018;
    RUNTIME_DICT_KEY_NOT_FOUND = 4019;
    RUNTIME_VM_SHUTDOWN = 4020;
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS = 4021;

    /// `Std.Str.Format`: specifier count does not match argument list, or a type does not match its specifier.
    RUNTIME_FORMAT_MISMATCH = 4022;

    /// `Std.Test` assertion or explicit `Fail` call.
    RUNTIME_TEST_ASSERTION_FAILED = 4023;

    /// Recording capture hit a host effect that is not replayable.
    RUNTIME_RECORDING_UNSUPPORTED_EFFECT = 4024;

    /// The operating system could not supply random bytes.
    RUNTIME_RANDOM_SOURCE_FAILURE = 4025;
});

define_codes!(PROJECT_ALLOCATED_CODES => {
    /// Reading a project source file failed before a source position was available.
    PROJECT_SOURCE_READ_FAILED = 5001;
    /// Source bytes are not valid UTF-8; no scalar source position is available.
    PROJECT_SOURCE_INVALID_UTF8 = 5002;
    /// A `.fpasprj` or `.fpasworkspace` manifest could not be read.
    PROJECT_MANIFEST_READ_FAILED = 5003;
    /// A manifest is not valid TOML or does not match the manifest schema.
    PROJECT_MANIFEST_SYNTAX_INVALID = 5004;
    /// A manifest field has an empty, unknown, or disallowed value.
    PROJECT_MANIFEST_VALUE_INVALID = 5005;
    /// A manifest path is missing, not a file, has the wrong extension, or cannot be resolved.
    PROJECT_PATH_INVALID = 5006;
    /// A source glob pattern is invalid, cannot be evaluated, or matches no files.
    PROJECT_PATTERN_INVALID = 5007;
    /// A manifest lists the same entry more than once.
    PROJECT_DUPLICATE_ENTRY = 5008;
    /// `dependencies.projects` references a project that is not a library.
    PROJECT_DEPENDENCY_NOT_LIBRARY = 5009;
    /// Library projects depend on each other in a cycle.
    PROJECT_DEPENDENCY_CYCLE = 5010;
    /// One source file belongs to more than one project.
    PROJECT_SOURCE_OWNERSHIP_CONFLICT = 5011;
    /// A source file declares `program` where a `unit` is required, or the reverse.
    PROJECT_UNIT_KIND_MISMATCH = 5012;
    /// A unit name uses a namespace reserved for or required by the standard library.
    PROJECT_UNIT_NAMESPACE_INVALID = 5013;
    /// Two source files declare the same unit name.
    PROJECT_DUPLICATE_UNIT = 5014;
    /// A `uses` clause or `[exports].units` names a unit that no source declares.
    PROJECT_UNKNOWN_UNIT = 5015;
    /// A unit is imported across a library boundary without being exported.
    PROJECT_UNIT_NOT_EXPORTED = 5016;
    /// Units depend on each other in a cycle.
    PROJECT_UNIT_CYCLE = 5017;
    /// The project links more source files than source IDs can address.
    PROJECT_SOURCE_LIMIT_EXCEEDED = 5018;
    /// A source file no longer matches the snapshot the project graph was built from.
    PROJECT_SOURCE_CHANGED = 5019;
    /// A directory needed for project or workspace discovery could not be read.
    PROJECT_DIRECTORY_READ_FAILED = 5020;
    /// Project or workspace discovery found no candidate or more than one candidate.
    PROJECT_DISCOVERY_FAILED = 5021;
    /// `[dependencies].workspace` names no workspace member project.
    PROJECT_UNKNOWN_WORKSPACE_DEPENDENCY = 5022;
    /// The standard-library directory or manifest violates the trusted library rules.
    PROJECT_STANDARD_LIBRARY_INVALID = 5023;
    /// Reading, locking, writing or replacing a compiled-unit or program artifact failed.
    BUILD_ARTIFACT_IO_FAILED = 5024;
    /// A compiled interface, object or program image could not be encoded or validated.
    BUILD_ARTIFACT_ENCODING_FAILED = 5025;
    /// A compiled object is malformed or inconsistent with the objects it is linked with.
    LINK_INVALID_OBJECT = 5026;
    /// The root object of a program has no entry function.
    LINK_MISSING_PROGRAM_ENTRY = 5027;
    /// Two linked objects define the same canonical symbol.
    LINK_DUPLICATE_DEFINITION = 5028;
    /// Linked objects disagree on a record or enum layout, or a variant is missing.
    LINK_INCOMPATIBLE_LAYOUT = 5029;
    /// No linked object defines a required public symbol.
    LINK_UNRESOLVED_IMPORT = 5030;
    /// An object imports a definition that is private to another object.
    LINK_PRIVATE_IMPORT = 5031;
    /// An import resolves to a definition of the wrong kind or an incompatible ABI.
    LINK_INCOMPATIBLE_IMPORT = 5032;
    /// A linked table or address exceeds its fixed-width limit.
    LINK_LIMIT_EXCEEDED = 5033;
    /// The linked executable failed final bytecode verification.
    LINK_INVALID_EXECUTABLE = 5034;
    /// Warning: a source file matched more than once; the first occurrence was kept.
    PROJECT_DUPLICATE_SOURCE_FILE = 5035;
    /// Warning: a `program` source outside the entry rules was skipped.
    PROJECT_PROGRAM_SOURCE_SKIPPED = 5036;
    /// Command-line arguments are invalid, duplicated, or incomplete.
    CLI_ARGUMENTS_INVALID = 5037;
    /// The selected input kind cannot be used by the requested command.
    CLI_INPUT_UNSUPPORTED = 5038;
    /// A command could not write its promised output or output files.
    CLI_OUTPUT_FAILED = 5039;
    /// A test's standard output differs from its `.expect.stdout` sidecar.
    TEST_STDOUT_MISMATCH = 5040;
    /// A test exceeded its wall-clock timeout.
    TEST_TIMED_OUT = 5041;
    /// The test runner could not prepare, isolate, or finish a test or hook.
    TEST_RUNNER_FAILED = 5042;
});

define_codes!(INTERNAL_ALLOCATED_CODES => {
    INTERNAL_COMPILER_INVARIANT_FAILURE = 9001;
    INTERNAL_VM_INVARIANT_FAILURE = 9002;
    /// Project graph or build orchestration reached an inconsistent state.
    INTERNAL_PROJECT_INVARIANT_FAILURE = 9003;
});

#[cfg(test)]
const ALL_CODE_INVENTORIES: &[&[DiagnosticCode]] = &[
    LEX_ALLOCATED_CODES,
    PARSE_ALLOCATED_CODES,
    SEMA_ALLOCATED_CODES,
    COMPILE_ALLOCATED_CODES,
    RUNTIME_ALLOCATED_CODES,
    PROJECT_ALLOCATED_CODES,
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
        for code in PROJECT_ALLOCATED_CODES {
            assert_eq!(code.stage(), DiagnosticStage::Project);
        }
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
