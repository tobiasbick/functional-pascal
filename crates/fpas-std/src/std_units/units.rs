/// Root namespace segment for standard units.
pub const STD_ROOT_SEGMENT: &str = "Std";

macro_rules! standard_unit_names {
    ($($(#[$attr:meta])* $name:ident = $value:literal;)+) => {
        $(
            $(#[$attr])*
            #[doc = concat!("Canonical name of the `", $value, "` standard unit.")]
            pub const $name: &str = $value;
        )+
    };
}

standard_unit_names! {
    STD_UNIT_ARGS = "Std.Args";
    STD_UNIT_ENV = "Std.Env";
    STD_UNIT_PROC = "Std.Proc";
    STD_UNIT_PATH = "Std.Path";
    STD_UNIT_FS = "Std.Fs";
    STD_UNIT_TIME = "Std.Time";
    STD_UNIT_VERSION = "Std.Version";
    STD_UNIT_CONSOLE = "Std.Console";
    STD_UNIT_STR = "Std.Str";
    STD_UNIT_CONV = "Std.Conv";
    STD_UNIT_PARSE = "Std.Parse";
    STD_UNIT_MATH = "Std.Math";
    STD_UNIT_NET = "Std.Net";
    STD_UNIT_RANDOM = "Std.Random";
    STD_UNIT_ARRAY = "Std.Array";
    STD_UNIT_RESULT = "Std.Result";
    STD_UNIT_OPTION = "Std.Option";
    STD_UNIT_TASK = "Std.Task";
    STD_UNIT_DICT = "Std.Dict";
    STD_UNIT_JSON = "Std.Json";
    STD_UNIT_TOML = "Std.Toml";
    STD_UNIT_TUI = "Std.Tui";
    STD_UNIT_TEST = "Std.Test";
}

/// Standard units supplied entirely by compiler, VM, or runtime intrinsics.
pub const STD_UNITS_INTRINSIC: &[&str] = &[
    STD_UNIT_ARGS,
    STD_UNIT_ENV,
    STD_UNIT_PROC,
    STD_UNIT_PATH,
    STD_UNIT_FS,
    STD_UNIT_TIME,
    STD_UNIT_CONSOLE,
    STD_UNIT_STR,
    STD_UNIT_CONV,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
    STD_UNIT_NET,
    STD_UNIT_RANDOM,
    STD_UNIT_ARRAY,
    STD_UNIT_RESULT,
    STD_UNIT_OPTION,
    STD_UNIT_TASK,
    STD_UNIT_DICT,
    STD_UNIT_JSON,
    STD_UNIT_TOML,
    STD_UNIT_TEST,
];

/// Standard units recognized by tooling, including source-defined units.
pub const STD_UNITS_KNOWN: &[&str] = &[
    STD_UNIT_ARGS,
    STD_UNIT_ENV,
    STD_UNIT_PROC,
    STD_UNIT_PATH,
    STD_UNIT_FS,
    STD_UNIT_TIME,
    STD_UNIT_CONSOLE,
    STD_UNIT_STR,
    STD_UNIT_CONV,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
    STD_UNIT_NET,
    STD_UNIT_RANDOM,
    STD_UNIT_ARRAY,
    STD_UNIT_RESULT,
    STD_UNIT_OPTION,
    STD_UNIT_TASK,
    STD_UNIT_DICT,
    STD_UNIT_JSON,
    STD_UNIT_TOML,
    STD_UNIT_TUI,
    STD_UNIT_TEST,
];

#[cfg(test)]
mod tests {
    use super::{STD_UNIT_CONSOLE, STD_UNIT_TUI, STD_UNIT_VERSION, STD_UNITS_INTRINSIC};

    #[test]
    fn intrinsic_units_exclude_source_defined_units() {
        assert!(STD_UNITS_INTRINSIC.contains(&STD_UNIT_CONSOLE));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_TUI));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_VERSION));
    }
}
