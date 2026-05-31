//! Standard unit registry and lookup helpers for `Std.*`.
//!
//! **Documentation:** `docs/pascal/std/README.md` and the unit files under `docs/pascal/std/` (from the repository root).

mod symbols;
mod units;

pub use symbols::std_symbols;
pub use units::{
    STD_ROOT_SEGMENT, STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV,
    STD_UNIT_DICT, STD_UNIT_ENV, STD_UNIT_FS, STD_UNIT_GRAPH, STD_UNIT_JSON, STD_UNIT_MATH,
    STD_UNIT_OPTION, STD_UNIT_PARSE, STD_UNIT_PATH, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_STR,
    STD_UNIT_TASK, STD_UNIT_TIME, STD_UNIT_TUI, STD_UNITS_KNOWN,
};

use symbols::groups::{
    STD_ARGS_SYMBOLS, STD_ARRAY_SYMBOLS, STD_CONSOLE_SYMBOLS, STD_CONV_SYMBOLS, STD_DICT_SYMBOLS,
    STD_ENV_SYMBOLS, STD_FS_SYMBOLS, STD_GRAPH_SYMBOLS, STD_JSON_SYMBOLS, STD_MATH_SYMBOLS,
    STD_OPTION_SYMBOLS, STD_PARSE_SYMBOLS, STD_PATH_SYMBOLS, STD_RANDOM_SYMBOLS,
    STD_RESULT_SYMBOLS, STD_STR_SYMBOLS, STD_TASK_SYMBOLS, STD_TIME_SYMBOLS, STD_TUI_SYMBOLS,
};

pub fn is_std_root_segment(segment: &str) -> bool {
    segment.eq_ignore_ascii_case(STD_ROOT_SEGMENT)
}

pub fn canonical_std_unit_from_tail(tail: &str) -> Option<&'static str> {
    match tail.to_ascii_lowercase().as_str() {
        "args" => Some(STD_UNIT_ARGS),
        "env" => Some(STD_UNIT_ENV),
        "path" => Some(STD_UNIT_PATH),
        "fs" => Some(STD_UNIT_FS),
        "time" => Some(STD_UNIT_TIME),
        "console" => Some(STD_UNIT_CONSOLE),
        "tui" => Some(STD_UNIT_TUI),
        "graph" => Some(STD_UNIT_GRAPH),
        "str" => Some(STD_UNIT_STR),
        "conv" => Some(STD_UNIT_CONV),
        "parse" => Some(STD_UNIT_PARSE),
        "math" => Some(STD_UNIT_MATH),
        "random" => Some(STD_UNIT_RANDOM),
        "array" => Some(STD_UNIT_ARRAY),
        "result" => Some(STD_UNIT_RESULT),
        "option" => Some(STD_UNIT_OPTION),
        "task" => Some(STD_UNIT_TASK),
        "dict" => Some(STD_UNIT_DICT),
        "json" => Some(STD_UNIT_JSON),
        _ => None,
    }
}

pub fn canonical_std_unit_from_segments(root: &str, tail: &str) -> Option<&'static str> {
    if !is_std_root_segment(root) {
        return None;
    }

    canonical_std_unit_from_tail(tail)
}

pub fn std_unit_symbols(unit: &str) -> &'static [&'static str] {
    match unit {
        STD_UNIT_ARGS => STD_ARGS_SYMBOLS,
        STD_UNIT_ENV => STD_ENV_SYMBOLS,
        STD_UNIT_PATH => STD_PATH_SYMBOLS,
        STD_UNIT_FS => STD_FS_SYMBOLS,
        STD_UNIT_TIME => STD_TIME_SYMBOLS,
        STD_UNIT_CONSOLE => STD_CONSOLE_SYMBOLS,
        STD_UNIT_TUI => STD_TUI_SYMBOLS,
        STD_UNIT_GRAPH => STD_GRAPH_SYMBOLS,
        STD_UNIT_STR => STD_STR_SYMBOLS,
        STD_UNIT_CONV => STD_CONV_SYMBOLS,
        STD_UNIT_PARSE => STD_PARSE_SYMBOLS,
        STD_UNIT_MATH => STD_MATH_SYMBOLS,
        STD_UNIT_RANDOM => STD_RANDOM_SYMBOLS,
        STD_UNIT_ARRAY => STD_ARRAY_SYMBOLS,
        STD_UNIT_RESULT => STD_RESULT_SYMBOLS,
        STD_UNIT_OPTION => STD_OPTION_SYMBOLS,
        STD_UNIT_TASK => STD_TASK_SYMBOLS,
        STD_UNIT_DICT => STD_DICT_SYMBOLS,
        STD_UNIT_JSON => STD_JSON_SYMBOLS,
        _ => &[],
    }
}

pub fn std_units_list_for_hint() -> String {
    STD_UNITS_KNOWN.join(", ")
}
