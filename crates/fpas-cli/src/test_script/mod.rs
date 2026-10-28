//! Sidecar TOML scripts for scripted test input.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

mod apply;
mod parse;

pub use parse::{ScriptEvent, ScriptFile, load_script, parse_script_text, sidecar_path_for_test};

/// Parses a script file and pushes its events into VM input queues.
pub fn apply_script_to_vm(vm: &mut fpas_runtime::execution::Vm, script: &ScriptFile) {
    apply::apply_script(vm, script);
}

#[cfg(test)]
mod tests;
