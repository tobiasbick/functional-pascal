//! Lowers standard-library calls and intrinsics to VM operations.
//!
//! **Documentation:** `docs/pascal/11-stdlib.md`, `docs/pascal/std/README.md` (from the repository root).

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
mod parse;
mod path;
mod random;
mod result_option;
mod str_ops;
mod support;
mod task;
mod time;
mod tui;

use crate::error::CompileError;
use fpas_bytecode::SourceLocation;
use fpas_parser::Expr;

use super::Compiler;

impl Compiler {
    /// Returns `true` if this was a standard-library call (emitted here).
    pub(super) fn compile_std_library_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        if self.compile_args_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_env_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_path_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_fs_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_console_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_graph_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_str_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_conv_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_parse_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_math_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_random_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_array_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_dict_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_json_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_result_option_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_task_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_time_call(name, args, location)? {
            return Ok(true);
        }
        Ok(false)
    }
}
