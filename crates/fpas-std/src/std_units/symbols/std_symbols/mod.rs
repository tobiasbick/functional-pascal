macro_rules! std_console {
    ($suffix:literal) => {
        concat!("Std.Console.", $suffix)
    };
}
macro_rules! std_args {
    ($suffix:literal) => {
        concat!("Std.Args.", $suffix)
    };
}
macro_rules! std_env {
    ($suffix:literal) => {
        concat!("Std.Env.", $suffix)
    };
}
macro_rules! std_proc {
    ($suffix:literal) => {
        concat!("Std.Proc.", $suffix)
    };
}
macro_rules! std_path {
    ($suffix:literal) => {
        concat!("Std.Path.", $suffix)
    };
}
macro_rules! std_fs {
    ($suffix:literal) => {
        concat!("Std.Fs.", $suffix)
    };
}
macro_rules! std_time {
    ($suffix:literal) => {
        concat!("Std.Time.", $suffix)
    };
}
macro_rules! std_graph {
    ($suffix:literal) => {
        concat!("Std.Graph.", $suffix)
    };
}
macro_rules! std_str {
    ($suffix:literal) => {
        concat!("Std.Str.", $suffix)
    };
}
macro_rules! std_conv {
    ($suffix:literal) => {
        concat!("Std.Conv.", $suffix)
    };
}
macro_rules! std_parse {
    ($suffix:literal) => {
        concat!("Std.Parse.", $suffix)
    };
}
macro_rules! std_math {
    ($suffix:literal) => {
        concat!("Std.Math.", $suffix)
    };
}
macro_rules! std_random {
    ($suffix:literal) => {
        concat!("Std.Random.", $suffix)
    };
}
macro_rules! std_array {
    ($suffix:literal) => {
        concat!("Std.Array.", $suffix)
    };
}
macro_rules! std_result {
    ($suffix:literal) => {
        concat!("Std.Result.", $suffix)
    };
}
macro_rules! std_option {
    ($suffix:literal) => {
        concat!("Std.Option.", $suffix)
    };
}
macro_rules! std_task {
    ($suffix:literal) => {
        concat!("Std.Task.", $suffix)
    };
}
macro_rules! std_dict {
    ($suffix:literal) => {
        concat!("Std.Dict.", $suffix)
    };
}
macro_rules! std_json {
    ($suffix:literal) => {
        concat!("Std.Json.", $suffix)
    };
}
macro_rules! std_toml {
    ($suffix:literal) => {
        concat!("Std.Toml.", $suffix)
    };
}
macro_rules! std_test {
    ($suffix:literal) => {
        concat!("Std.Test.", $suffix)
    };
}

mod args;
mod array;
mod console;
mod conv;
mod dict;
mod env;
mod fs;
mod graph;
mod json;
mod math;
mod option;
mod parse;
mod path;
mod proc;
mod random;
mod result;
mod string;
mod task;
mod test;
mod time;
mod toml;

pub use args::*;
pub use array::*;
pub use console::*;
pub use conv::*;
pub use dict::*;
pub use env::*;
pub use fs::*;
pub use graph::*;
pub use json::*;
pub use math::*;
pub use option::*;
pub use parse::*;
pub use path::*;
pub use proc::*;
pub use random::*;
pub use result::*;
pub use string::*;
pub use task::*;
pub use test::*;
pub use time::*;
pub use toml::*;
