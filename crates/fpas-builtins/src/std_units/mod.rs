//! Standard unit registry and lookup helpers for `Std.*`.
//!
//! **Documentation:** `docs/pascal/std/README.md` and the unit files under `docs/pascal/std/` (from the repository root).

mod symbols;
mod units;

pub use symbols::std_symbols;
pub use units::{
    STD_ROOT_SEGMENT, STD_UNIT_ACP, STD_UNIT_ACP_EXPERIMENTAL, STD_UNIT_ACP_HOST,
    STD_UNIT_ACP_TYPES, STD_UNIT_ACP_V2, STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_BITS,
    STD_UNIT_CALENDAR, STD_UNIT_CLI, STD_UNIT_CONSOLE, STD_UNIT_CRYPTO, STD_UNIT_DICT,
    STD_UNIT_ENCODING, STD_UNIT_ENV, STD_UNIT_ERRORS, STD_UNIT_FORMAT, STD_UNIT_FS, STD_UNIT_JSON,
    STD_UNIT_MATH, STD_UNIT_MCP, STD_UNIT_MCP_TYPES, STD_UNIT_NET, STD_UNIT_OPTION, STD_UNIT_PARSE,
    STD_UNIT_PATH, STD_UNIT_PROC, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_SERVER, STD_UNIT_STR,
    STD_UNIT_TASK, STD_UNIT_TEST, STD_UNIT_TIME, STD_UNIT_TOKENIZER, STD_UNIT_TOML, STD_UNIT_TUI,
    STD_UNIT_VERSION, STD_UNITS_INTRINSIC, STD_UNITS_KNOWN,
};

use symbols::{
    STD_ACP_HOST_SYMBOLS, STD_ACP_SYMBOLS, STD_ACP_TYPES_SYMBOLS, STD_ACP_V2_SYMBOLS,
    STD_ARGS_SYMBOLS, STD_ARRAY_SYMBOLS, STD_BITS_SYMBOLS, STD_CONSOLE_SYMBOLS, STD_CRYPTO_SYMBOLS,
    STD_DICT_SYMBOLS, STD_ENCODING_SYMBOLS, STD_ENV_SYMBOLS, STD_FORMAT_SYMBOLS, STD_FS_SYMBOLS,
    STD_JSON_SYMBOLS, STD_MATH_SYMBOLS, STD_MCP_SYMBOLS, STD_MCP_TYPES_SYMBOLS, STD_NET_SYMBOLS,
    STD_OPTION_SYMBOLS, STD_PARSE_SYMBOLS, STD_PATH_SYMBOLS, STD_PROC_SYMBOLS, STD_RANDOM_SYMBOLS,
    STD_RESULT_SYMBOLS, STD_SERVER_SYMBOLS, STD_STR_SYMBOLS, STD_TASK_SYMBOLS, STD_TEST_SYMBOLS,
    STD_TIME_SYMBOLS, STD_TOKENIZER_SYMBOLS, STD_TOML_SYMBOLS,
};

/// Returns whether `segment` names the case-insensitive `Std` root namespace.
pub fn is_std_root_segment(segment: &str) -> bool {
    segment.eq_ignore_ascii_case(STD_ROOT_SEGMENT)
}

/// Whether a former type-helper unit is now owned by import-free native operations.
#[must_use]
pub fn is_retired_type_helper_unit(name: &str) -> bool {
    [
        STD_UNIT_STR,
        STD_UNIT_ARRAY,
        STD_UNIT_DICT,
        STD_UNIT_OPTION,
        STD_UNIT_RESULT,
    ]
    .iter()
    .any(|unit| unit.eq_ignore_ascii_case(name))
}

/// Resolves a case-insensitive unit tail such as `console` to its canonical name.
pub fn canonical_std_unit_from_tail(tail: &str) -> Option<&'static str> {
    STD_UNITS_KNOWN
        .iter()
        .copied()
        .chain(std::iter::once(STD_UNIT_VERSION))
        .find(|unit| {
            unit.strip_prefix("Std.")
                .is_some_and(|name| tail.eq_ignore_ascii_case(name))
        })
}

/// Resolves separate root and tail segments to a canonical standard-unit name.
pub fn canonical_std_unit_from_segments(root: &str, tail: &str) -> Option<&'static str> {
    if !is_std_root_segment(root) {
        return None;
    }

    canonical_std_unit_from_tail(tail)
}

/// Returns the intrinsic symbol names registered for a canonical standard unit.
pub fn std_unit_symbols(unit: &str) -> &'static [&'static str] {
    match unit {
        STD_UNIT_TOKENIZER => STD_TOKENIZER_SYMBOLS,
        STD_UNIT_MCP => STD_MCP_SYMBOLS,
        STD_UNIT_MCP_TYPES => STD_MCP_TYPES_SYMBOLS,
        STD_UNIT_ACP => STD_ACP_SYMBOLS,
        STD_UNIT_ACP_TYPES => STD_ACP_TYPES_SYMBOLS,
        STD_UNIT_ACP_V2 => STD_ACP_V2_SYMBOLS,
        STD_UNIT_ACP_HOST => STD_ACP_HOST_SYMBOLS,
        STD_UNIT_ERRORS => &[],
        STD_UNIT_SERVER => STD_SERVER_SYMBOLS,
        STD_UNIT_ARGS => STD_ARGS_SYMBOLS,
        STD_UNIT_ENV => STD_ENV_SYMBOLS,
        STD_UNIT_PROC => STD_PROC_SYMBOLS,
        STD_UNIT_PATH => STD_PATH_SYMBOLS,
        STD_UNIT_FS => STD_FS_SYMBOLS,
        STD_UNIT_TIME => STD_TIME_SYMBOLS,
        STD_UNIT_VERSION => &[],
        STD_UNIT_CONSOLE => STD_CONSOLE_SYMBOLS,
        STD_UNIT_STR => STD_STR_SYMBOLS,
        STD_UNIT_FORMAT => STD_FORMAT_SYMBOLS,
        STD_UNIT_CRYPTO => STD_CRYPTO_SYMBOLS,
        STD_UNIT_ENCODING => STD_ENCODING_SYMBOLS,
        STD_UNIT_PARSE => STD_PARSE_SYMBOLS,
        STD_UNIT_MATH => STD_MATH_SYMBOLS,
        STD_UNIT_BITS => STD_BITS_SYMBOLS,
        STD_UNIT_NET => STD_NET_SYMBOLS,
        STD_UNIT_RANDOM => STD_RANDOM_SYMBOLS,
        STD_UNIT_ARRAY => STD_ARRAY_SYMBOLS,
        STD_UNIT_RESULT => STD_RESULT_SYMBOLS,
        STD_UNIT_OPTION => STD_OPTION_SYMBOLS,
        STD_UNIT_TASK => STD_TASK_SYMBOLS,
        STD_UNIT_DICT => STD_DICT_SYMBOLS,
        STD_UNIT_JSON => STD_JSON_SYMBOLS,
        STD_UNIT_TOML => STD_TOML_SYMBOLS,
        STD_UNIT_TUI => &[],
        STD_UNIT_TEST => STD_TEST_SYMBOLS,
        _ => &[],
    }
}

/// Formats all recognized standard units for diagnostic hints.
pub fn std_units_list_for_hint() -> String {
    STD_UNITS_KNOWN.join(", ")
}
