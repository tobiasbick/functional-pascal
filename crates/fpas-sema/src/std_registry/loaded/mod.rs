mod array;
mod channel_task;
mod console;
mod conv;
mod dict;
mod math;
mod result_option;
mod str_ops;
mod tui;

use crate::check::Checker;
use fpas_std::{
    STD_UNIT_ARRAY, STD_UNIT_CONSOLE, STD_UNIT_CONV, STD_UNIT_DICT, STD_UNIT_MATH, STD_UNIT_OPTION,
    STD_UNIT_RESULT, STD_UNIT_STR, STD_UNIT_TASK, STD_UNIT_TUI, STD_UNITS_KNOWN,
};

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
        STD_UNIT_CONSOLE => console::register_std_console(checker),
        STD_UNIT_STR => str_ops::register_std_str(checker),
        STD_UNIT_CONV => conv::register_std_conv(checker),
        STD_UNIT_MATH => math::register_std_math(checker),
        STD_UNIT_ARRAY => array::register_std_array(checker),
        STD_UNIT_RESULT => result_option::register_std_result(checker),
        STD_UNIT_OPTION => result_option::register_std_option(checker),
        STD_UNIT_TASK => channel_task::register_std_task(checker),
        STD_UNIT_DICT => dict::register_std_dict(checker),
        STD_UNIT_TUI => tui::register_std_tui(checker),
        _ => {}
    }
}
