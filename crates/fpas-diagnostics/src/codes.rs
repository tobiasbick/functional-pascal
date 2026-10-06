//! Stable diagnostic code catalog.
//!
//! Numeric ranges (see also [`DiagnosticCode::stage`]):
//! - **Lex:** `FP1000`–`FP1999`
//! - **Parse:** `FP2000`–`FP2999`
//! - **Sema:** `FP3000`–`FP3999`
//! - **Compile:** `FP4000`–`FP4099`
//! - **Project/build/linker/CLI:** `FP4100`–`FP4999`
//! - **Runtime:** `FP5000`–`FP5999` (reserved gap: `FP5017`)
//! - **Internal:** `FP9001`–`FP9999` and any other unassigned value
//!
//! Extension workflow:
//! 1. Add a new named `pub const` in the correct stage block below.
//! 2. Use the next free numeric value inside that stage range.
//! 3. Re-run `cargo test -p fpas-diagnostics` (uniqueness and stage-range tests).
//! 4. Document a wrong and corrected example in `docs/pascal/tools/diagnostics.md`.

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
    LEX_UNEXPECTED_CHARACTER = 1001;
    LEX_UNTERMINATED_STRING_LITERAL = 1004;
    LEX_INVALID_CHARACTER_CODE_LITERAL = 1005;
    LEX_INVALID_HEXADECIMAL_LITERAL = 1006;
    LEX_INTEGER_LITERAL_OVERFLOW = 1007;
    LEX_REAL_LITERAL_OVERFLOW = 1008;
    LEX_INVALID_NUMERIC_EXPONENT = 1009;

    /// Lexer: `{$...}` is invalid source syntax.
    LEX_COMPILER_DIRECTIVE_NOT_SUPPORTED = 1010;
    /// Lexer: `__` or other invalid `_` placement inside a numeric literal.
    LEX_INVALID_DIGIT_SEPARATOR = 1011;
    /// Lexer: non-ASCII letter/digit in an identifier (ASCII letters, digits, `_` only).
    LEX_NON_ASCII_IN_IDENTIFIER = 1012;
    /// Lexer: a comment-like form other than `//` was used.
    LEX_INVALID_COMMENT_FORM = 1013;
});

define_codes!(PARSE_ALLOCATED_CODES => {
    PARSE_EXPECTED_TOKEN = 2001;
    PARSE_EXPECTED_IDENTIFIER = 2002;
    PARSE_INVALID_STATEMENT_START = 2003;
    PARSE_EXPECTED_TO_OR_DOWNTO = 2004;
    PARSE_EXPECTED_EXPRESSION = 2005;
    PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM = 2006;

    /// The `public` visibility modifier was used outside a `unit` file.
    PARSE_INVALID_VISIBILITY = 2007;
    /// `static` used outside a supported static record routine.
    PARSE_INVALID_STATIC_PLACEMENT = 2008;
    /// Parser recursion exceeded the compiler's shared nesting budget.
    ///
    /// **Documentation:** `docs/pascal/program-structure/cli.md` (Checking without running)
    PARSE_NESTING_LIMIT_EXCEEDED = 2009;
    /// A record update expression contains no field assignments.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
    PARSE_EMPTY_RECORD_UPDATE = 2010;
    /// An event declaration places its `write` accessor before its `read` accessor.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    PARSE_INVALID_EVENT_ACCESSOR_ORDER = 2011;
    /// An enum variant declares an empty associated-data field list.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    PARSE_EMPTY_ENUM_FIELD_LIST = 2012;
    /// An enum variant field list ends with a trailing semicolon.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    PARSE_TRAILING_ENUM_FIELD_SEPARATOR = 2013;
    /// Comma-separated or grouped formal parameters require individually typed declarations.
    ///
    /// Documentation: `docs/pascal/tools/diagnostics.md` (FP2014).
    PARSE_INVALID_PARAMETER_SEPARATOR = 2014;
});

define_codes!(SEMA_ALLOCATED_CODES => {
    SEMA_UNKNOWN_TYPE = 3001;
    SEMA_DUPLICATE_DECLARATION = 3002;
    SEMA_UNKNOWN_NAME = 3003;
    SEMA_AMBIGUOUS_IMPORTED_NAME = 3004;
    SEMA_IMMUTABLE_ASSIGNMENT = 3005;
    SEMA_TYPE_MISMATCH = 3006;
    SEMA_WRONG_ARGUMENT_COUNT = 3007;
    SEMA_NON_BOOLEAN_CONDITION = 3008;
    SEMA_INVALID_PANIC_ARGUMENT = 3009;
    SEMA_INVALID_BREAK_OR_CONTINUE_PLACEMENT = 3010;
    SEMA_NON_EXHAUSTIVE_CASE = 3011;
    SEMA_ENUM_FIELD_COUNT_MISMATCH = 3012;
    SEMA_CONSTRAINT_VIOLATION = 3013;
    SEMA_NON_CONSTANT_EXPRESSION = 3014;

    /// A required record field (without a default value) is missing from a record literal.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md` (Default field values)
    SEMA_MISSING_RECORD_FIELD = 3015;

    /// A task-bound callable (mutable captures) cannot cross a task boundary.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    SEMA_TASK_BOUND_CALLABLE = 3016;

    /// A non-public record member is accessed outside its declaring unit.
    ///
    /// **Documentation:** `docs/pascal/language/types/records.md`
    SEMA_PRIVATE_RECORD_MEMBER = 3017;

    /// An implicit enum backing value would exceed the signed 64-bit integer range.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    SEMA_ENUM_BACKING_VALUE_EXHAUSTED = 3018;

    /// A public unit declaration refers to a type that is private to that unit.
    ///
    /// **Documentation:** `docs/pascal/program-structure/visibility.md`
    SEMA_PRIVATE_TYPE_IN_PUBLIC_SIGNATURE = 3019;
    /// Explicit discard requires a value rather than a procedure result.
    /// Documentation: `docs/pascal/language/functions/discard.md`.
    SEMA_DISCARD_REQUIRES_VALUE = 3020;
    /// Explicit discard could lose a task handle or unverified callable captures.
    /// Documentation: `docs/pascal/language/functions/discard.md`.
    SEMA_UNSAFE_DISCARD = 3021;
});

define_codes!(COMPILE_ALLOCATED_CODES => {
    COMPILE_INVALID_DESIGNATOR_BASE = 4001;
    COMPILE_INVALID_ASSIGNMENT_TARGET = 4002;
    COMPILE_INTRINSIC_ARITY_MISMATCH = 4003;
    COMPILE_UNSUPPORTED_INTRINSIC_LOWERING_CASE = 4004;
    COMPILE_INVALID_MUTABLE_ARRAY_LOWERING_TARGET = 4005;
    COMPILE_INVALID_GO_EXPRESSION = 4006;
    COMPILE_BYTECODE_OPERAND_OVERFLOW = 4007;
});

define_codes!(RUNTIME_ALLOCATED_CODES => {
    RUNTIME_DIVISION_BY_ZERO = 5001;
    RUNTIME_MODULO_BY_ZERO = 5002;
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS = 5003;
    RUNTIME_POP_FROM_EMPTY_ARRAY = 5004;
    RUNTIME_UNDEFINED_GLOBAL = 5005;
    RUNTIME_UNDEFINED_FUNCTION = 5006;
    RUNTIME_WRONG_CALL_ARITY = 5007;

    /// Operand has the wrong dynamic type for the operation (including std intrinsic argument checks).
    RUNTIME_VM_OPERAND_TYPE_MISMATCH = 5008;

    /// Intrinsic stack underflow, or an argument violates an intrinsic precondition (not a dynamic type mismatch).
    RUNTIME_INTRINSIC_STACK_STATE_ERROR = 5009;
    RUNTIME_PROGRAM_PANIC = 5010;
    RUNTIME_CONSOLE_INPUT_FAILURE = 5011;
    RUNTIME_NUMERIC_DOMAIN_ERROR = 5012;
    RUNTIME_CONVERSION_FAILURE = 5013;
    RUNTIME_CONSOLE_STATE_ERROR = 5014;
    RUNTIME_UNWRAP_FAILURE = 5015;

    /// A retained task was cancelled by the debugger without running its remaining body.
    RUNTIME_TASK_CANCELLED = 5016;
    // Reserved: 5017 (gap before task/runtime codes; do not reuse without audit).
    RUNTIME_INVALID_TASK = 5018;
    RUNTIME_DICT_KEY_NOT_FOUND = 5019;
    RUNTIME_VM_SHUTDOWN = 5020;
    RUNTIME_STRING_INDEX_OUT_OF_BOUNDS = 5021;

    /// `Std.Str.Format`: specifier count does not match argument list, or a type does not match its specifier.
    RUNTIME_FORMAT_MISMATCH = 5022;

    /// `Std.Test` assertion or explicit `Fail` call.
    RUNTIME_TEST_ASSERTION_FAILED = 5023;

    /// Recording capture hit a host effect that is not replayable.
    RUNTIME_RECORDING_UNSUPPORTED_EFFECT = 5024;

    /// The operating system could not supply random bytes.
    RUNTIME_RANDOM_SOURCE_FAILURE = 5025;
});

define_codes!(PROJECT_ALLOCATED_CODES => {
    /// Reading a project source file failed before a source position was available.
    PROJECT_SOURCE_READ_FAILED = 4101;
    /// Source bytes are not valid UTF-8; no scalar source position is available.
    PROJECT_SOURCE_INVALID_UTF8 = 4102;
    /// A `.fpasprj` or `.fpasworkspace` manifest could not be read.
    PROJECT_MANIFEST_READ_FAILED = 4103;
    /// A manifest is not valid TOML or does not match the manifest schema.
    PROJECT_MANIFEST_SYNTAX_INVALID = 4104;
    /// A manifest field has an empty, unknown, or disallowed value.
    PROJECT_MANIFEST_VALUE_INVALID = 4105;
    /// A manifest path is missing, not a file, has the wrong extension, or cannot be resolved.
    PROJECT_PATH_INVALID = 4106;
    /// A source glob pattern is invalid, cannot be evaluated, or matches no files.
    PROJECT_PATTERN_INVALID = 4107;
    /// A manifest lists the same entry more than once.
    PROJECT_DUPLICATE_ENTRY = 4108;
    /// `dependencies.projects` references a project that is not a library.
    PROJECT_DEPENDENCY_NOT_LIBRARY = 4109;
    /// Library projects depend on each other in a cycle.
    PROJECT_DEPENDENCY_CYCLE = 4110;
    /// One source file belongs to more than one project.
    PROJECT_SOURCE_OWNERSHIP_CONFLICT = 4111;
    /// A source file declares `program` where a `unit` is required, or the reverse.
    PROJECT_UNIT_KIND_MISMATCH = 4112;
    /// A unit name uses a namespace reserved for or required by the standard library.
    PROJECT_UNIT_NAMESPACE_INVALID = 4113;
    /// Two source files declare the same unit name.
    PROJECT_DUPLICATE_UNIT = 4114;
    /// A `uses` clause or `[exports].units` names a unit that no source declares.
    PROJECT_UNKNOWN_UNIT = 4115;
    /// A unit is imported across a library boundary without being exported.
    PROJECT_UNIT_NOT_EXPORTED = 4116;
    /// Units depend on each other in a cycle.
    PROJECT_UNIT_CYCLE = 4117;
    /// The project links more source files than source IDs can address.
    PROJECT_SOURCE_LIMIT_EXCEEDED = 4118;
    /// A source file no longer matches the snapshot the project graph was built from.
    PROJECT_SOURCE_CHANGED = 4119;
    /// A directory needed for project or workspace discovery could not be read.
    PROJECT_DIRECTORY_READ_FAILED = 4120;
    /// Project or workspace discovery found no candidate or more than one candidate.
    PROJECT_DISCOVERY_FAILED = 4121;
    /// `[dependencies].workspace` names no workspace member project.
    PROJECT_UNKNOWN_WORKSPACE_DEPENDENCY = 4122;
    /// The standard-library directory or manifest violates the trusted library rules.
    PROJECT_STANDARD_LIBRARY_INVALID = 4123;
    /// Reading, locking, writing or replacing a compiled-unit or program artifact failed.
    BUILD_ARTIFACT_IO_FAILED = 4124;
    /// A compiled interface, object or program image could not be encoded or validated.
    BUILD_ARTIFACT_ENCODING_FAILED = 4125;
    /// A compiled object is malformed or inconsistent with the objects it is linked with.
    LINK_INVALID_OBJECT = 4126;
    /// The root object of a program has no entry function.
    LINK_MISSING_PROGRAM_ENTRY = 4127;
    /// Two linked objects define the same canonical symbol.
    LINK_DUPLICATE_DEFINITION = 4128;
    /// Linked objects disagree on a record or enum layout, or a variant is missing.
    LINK_INCOMPATIBLE_LAYOUT = 4129;
    /// No linked object defines a required public symbol.
    LINK_UNRESOLVED_IMPORT = 4130;
    /// An object imports a definition that is private to another object.
    LINK_PRIVATE_IMPORT = 4131;
    /// An import resolves to a definition of the wrong kind or an incompatible ABI.
    LINK_INCOMPATIBLE_IMPORT = 4132;
    /// A linked table or address exceeds its fixed-width limit.
    LINK_LIMIT_EXCEEDED = 4133;
    /// The linked executable failed final bytecode verification.
    LINK_INVALID_EXECUTABLE = 4134;
    /// Warning: a source file matched more than once; the first occurrence was kept.
    PROJECT_DUPLICATE_SOURCE_FILE = 4135;
    /// Warning: a `program` source outside the entry rules was skipped.
    PROJECT_PROGRAM_SOURCE_SKIPPED = 4136;
    /// Command-line arguments are invalid, duplicated, or incomplete.
    CLI_ARGUMENTS_INVALID = 4137;
    /// The selected input kind cannot be used by the requested command.
    CLI_INPUT_UNSUPPORTED = 4138;
    /// A command could not write its promised output or output files.
    CLI_OUTPUT_FAILED = 4139;
    /// A test's standard output differs from its `.expect.stdout` sidecar.
    TEST_STDOUT_MISMATCH = 4140;
    /// A test exceeded its wall-clock timeout.
    TEST_TIMED_OUT = 4141;
    /// The test runner could not prepare, isolate, or finish a test or hook.
    TEST_RUNNER_FAILED = 4142;
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
mod tests;
