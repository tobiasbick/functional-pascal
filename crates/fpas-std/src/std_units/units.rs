pub const STD_ROOT_SEGMENT: &str = "Std";

pub const STD_UNIT_ARGS: &str = "Std.Args";
pub const STD_UNIT_ENV: &str = "Std.Env";
pub const STD_UNIT_PROC: &str = "Std.Proc";
pub const STD_UNIT_PATH: &str = "Std.Path";
pub const STD_UNIT_FS: &str = "Std.Fs";
pub const STD_UNIT_TIME: &str = "Std.Time";
/// Source-level version information supplied by the bundled standard library.
pub const STD_UNIT_VERSION: &str = "Std.Version";
pub const STD_UNIT_CONSOLE: &str = "Std.Console";
pub const STD_UNIT_STR: &str = "Std.Str";
pub const STD_UNIT_CONV: &str = "Std.Conv";
pub const STD_UNIT_PARSE: &str = "Std.Parse";
pub const STD_UNIT_MATH: &str = "Std.Math";
pub const STD_UNIT_RANDOM: &str = "Std.Random";
pub const STD_UNIT_ARRAY: &str = "Std.Array";
pub const STD_UNIT_RESULT: &str = "Std.Result";
pub const STD_UNIT_OPTION: &str = "Std.Option";
pub const STD_UNIT_TASK: &str = "Std.Task";
pub const STD_UNIT_DICT: &str = "Std.Dict";
pub const STD_UNIT_JSON: &str = "Std.Json";
pub const STD_UNIT_TOML: &str = "Std.Toml";
/// Source-level Model-Update-View terminal UI supplied by the bundled standard library.
pub const STD_UNIT_TUI: &str = "Std.Tui";
pub const STD_UNIT_GRAPH: &str = "Std.Graph";
pub const STD_UNIT_TEST: &str = "Std.Test";

/// Standard units supplied entirely by compiler, VM, or runtime intrinsics.
pub const STD_UNITS_INTRINSIC: &[&str] = &[
    STD_UNIT_ARGS,
    STD_UNIT_ENV,
    STD_UNIT_PROC,
    STD_UNIT_PATH,
    STD_UNIT_FS,
    STD_UNIT_TIME,
    STD_UNIT_CONSOLE,
    STD_UNIT_GRAPH,
    STD_UNIT_STR,
    STD_UNIT_CONV,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
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
    STD_UNIT_GRAPH,
    STD_UNIT_STR,
    STD_UNIT_CONV,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
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
