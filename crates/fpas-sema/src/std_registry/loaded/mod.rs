mod args;
mod array;
mod channel_task;
mod console;
mod conv;
mod dict;
mod env;
mod fs;
mod json;
mod math;
mod parse;
mod path;
mod proc;
mod random;
mod result_option;
mod str_ops;
mod test;
mod time;
mod toml;
mod type_registration;

use crate::check::Checker;
use fpas_std::{
    STD_UNIT_ARGS, STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV, STD_UNIT_DICT, STD_UNIT_ENV,
    STD_UNIT_FS, STD_UNIT_JSON, STD_UNIT_MATH, STD_UNIT_OPTION, STD_UNIT_PARSE, STD_UNIT_PATH,
    STD_UNIT_PROC, STD_UNIT_RANDOM, STD_UNIT_RESULT, STD_UNIT_STR, STD_UNIT_TASK, STD_UNIT_TEST,
    STD_UNIT_TIME, STD_UNIT_TOML, STD_UNIT_TUI, STD_UNITS_KNOWN,
};

const SOURCE_STD_UNIT_VERSION: &str = "Std.Version";

pub fn register_loaded_std(checker: &mut Checker) {
    for unit in STD_UNITS_KNOWN {
        if checker.loaded_std_units.contains(*unit) {
            register_single_std_unit(checker, unit);
        }
    }
}

/// Register symbols for one standard unit (idempotent if the unit was already registered).
pub fn register_single_std_unit(checker: &mut Checker, unit: &str) {
    match unit {
        STD_UNIT_ARGS => args::register_std_args(checker),
        STD_UNIT_ENV => env::register_std_env(checker),
        STD_UNIT_PROC => proc::register_std_proc(checker),
        STD_UNIT_PATH => path::register_std_path(checker),
        STD_UNIT_FS => fs::register_std_fs(checker),
        STD_UNIT_CONSOLE => console::register_std_console(checker),
        STD_UNIT_STR => str_ops::register_std_str(checker),
        STD_UNIT_CONV => conv::register_std_conv(checker),
        STD_UNIT_PARSE => parse::register_std_parse(checker),
        STD_UNIT_MATH => math::register_std_math(checker),
        STD_UNIT_RANDOM => random::register_std_random(checker),
        STD_UNIT_ARRAY => array::register_std_array(checker),
        STD_UNIT_RESULT => result_option::register_std_result(checker),
        STD_UNIT_OPTION => result_option::register_std_option(checker),
        STD_UNIT_TASK => channel_task::register_std_task(checker),
        STD_UNIT_TIME => time::register_std_time(checker),
        SOURCE_STD_UNIT_VERSION => {}
        STD_UNIT_DICT => dict::register_std_dict(checker),
        STD_UNIT_JSON => json::register_std_json(checker),
        STD_UNIT_TOML => toml::register_std_toml(checker),
        STD_UNIT_TUI => {}
        STD_UNIT_TEST => test::register_std_test(checker),
        _ => unreachable!(
            "register_single_std_unit: unhandled std unit `{unit}` — add a match arm for every entry in fpas_std::STD_UNITS_KNOWN"
        ),
    }
}
