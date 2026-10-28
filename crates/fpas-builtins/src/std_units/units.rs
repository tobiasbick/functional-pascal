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
    STD_UNIT_MCP = "Std.Ai.Mcp";
    STD_UNIT_MCP_TYPES = "Std.Ai.Mcp.Types";
    STD_UNIT_ACP = "Std.Ai.Acp";
    STD_UNIT_ACP_HOST = "Std.Ai.Acp.Host";
    STD_UNIT_ACP_EXPERIMENTAL = "Std.Ai.Acp.Experimental";
    STD_UNIT_ACP_V2 = "Std.Ai.Acp.V2";
    STD_UNIT_ACP_TYPES = "Std.Ai.Acp.Types";
    STD_UNIT_ERRORS = "Std.Os.Errors";
    STD_UNIT_SERVER = "Std.Net.Server";
    STD_UNIT_ARGS = "Std.Os.Args";
    STD_UNIT_CLI = "Std.Cli";
    STD_UNIT_CALENDAR = "Std.Time.Calendar";
    STD_UNIT_ENV = "Std.Os.Env";
    STD_UNIT_PROC = "Std.Os.Proc";
    STD_UNIT_PATH = "Std.Io.Path";
    STD_UNIT_FS = "Std.Io.Fs";
    STD_UNIT_TIME = "Std.Time";
    STD_UNIT_VERSION = "Std.Version";
    STD_UNIT_CONSOLE = "Std.Console";
    STD_UNIT_STR = "Std.Str";
    STD_UNIT_FORMAT = "Std.Text.Format";
    STD_UNIT_PARSE = "Std.Text.Parse";
    STD_UNIT_MATH = "Std.Math";
    STD_UNIT_BITS = "Std.Math.Bits";
    STD_UNIT_NET = "Std.Net.Sockets";
    STD_UNIT_RANDOM = "Std.Math.Random";
    STD_UNIT_CRYPTO = "Std.Crypto";
    STD_UNIT_ENCODING = "Std.Encoding";
    STD_UNIT_ARRAY = "Std.Arrays";
    STD_UNIT_RESULT = "Std.Results";
    STD_UNIT_OPTION = "Std.Options";
    STD_UNIT_TASK = "Std.Tasks";
    STD_UNIT_DICT = "Std.Dictionaries";
    STD_UNIT_JSON = "Std.Encoding.Json";
    STD_UNIT_TOML = "Std.Encoding.Toml";
    STD_UNIT_TUI = "Std.Ui.Terminal";
    STD_UNIT_TEST = "Std.Test";
}

/// Standard units supplied entirely by compiler, VM, or runtime intrinsics.
pub const STD_UNITS_INTRINSIC: &[&str] = &[
    STD_UNIT_MCP,
    STD_UNIT_MCP_TYPES,
    STD_UNIT_ACP,
    STD_UNIT_ACP_HOST,
    STD_UNIT_ACP_EXPERIMENTAL,
    STD_UNIT_ACP_V2,
    STD_UNIT_ACP_TYPES,
    STD_UNIT_ERRORS,
    STD_UNIT_SERVER,
    STD_UNIT_ARGS,
    STD_UNIT_ENV,
    STD_UNIT_PROC,
    STD_UNIT_PATH,
    STD_UNIT_FS,
    STD_UNIT_TIME,
    STD_UNIT_CONSOLE,
    STD_UNIT_FORMAT,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
    STD_UNIT_BITS,
    STD_UNIT_NET,
    STD_UNIT_RANDOM,
    STD_UNIT_CRYPTO,
    STD_UNIT_ENCODING,
    STD_UNIT_TASK,
    STD_UNIT_JSON,
    STD_UNIT_TOML,
    STD_UNIT_TEST,
];

/// Standard units recognized by tooling, including source-defined units.
pub const STD_UNITS_KNOWN: &[&str] = &[
    STD_UNIT_MCP,
    STD_UNIT_MCP_TYPES,
    STD_UNIT_ACP,
    STD_UNIT_ACP_HOST,
    STD_UNIT_ACP_EXPERIMENTAL,
    STD_UNIT_ACP_V2,
    STD_UNIT_ACP_TYPES,
    STD_UNIT_ERRORS,
    STD_UNIT_SERVER,
    STD_UNIT_ARGS,
    STD_UNIT_CLI,
    STD_UNIT_CALENDAR,
    STD_UNIT_ENV,
    STD_UNIT_PROC,
    STD_UNIT_PATH,
    STD_UNIT_FS,
    STD_UNIT_TIME,
    STD_UNIT_CONSOLE,
    STD_UNIT_FORMAT,
    STD_UNIT_PARSE,
    STD_UNIT_MATH,
    STD_UNIT_BITS,
    STD_UNIT_NET,
    STD_UNIT_RANDOM,
    STD_UNIT_CRYPTO,
    STD_UNIT_ENCODING,
    STD_UNIT_TASK,
    STD_UNIT_JSON,
    STD_UNIT_TOML,
    STD_UNIT_TUI,
    STD_UNIT_TEST,
];

#[cfg(test)]
mod tests {
    use super::{
        STD_UNIT_CALENDAR, STD_UNIT_CLI, STD_UNIT_CONSOLE, STD_UNIT_TUI, STD_UNIT_VERSION,
        STD_UNITS_INTRINSIC,
    };

    #[test]
    fn intrinsic_units_exclude_source_defined_units() {
        assert!(STD_UNITS_INTRINSIC.contains(&STD_UNIT_CONSOLE));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_TUI));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_CLI));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_CALENDAR));
        assert!(super::STD_UNITS_KNOWN.contains(&STD_UNIT_CALENDAR));
        assert!(!STD_UNITS_INTRINSIC.contains(&STD_UNIT_VERSION));
    }
}
