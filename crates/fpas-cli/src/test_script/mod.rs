//! Sidecar TOML scripts for scripted test input.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md)

mod apply;
mod graph;
mod input;
mod parse;

pub use parse::{ScriptConfig, ScriptFile, load_script, sidecar_path_for_test};

#[cfg(test)]
pub use parse::parse_script_text;

/// Parses a script file and pushes its events into VM input queues.
pub fn apply_script_to_vm(vm: &mut fpas_vm::Vm, script: &ScriptFile) -> Result<(), String> {
    apply::apply_script(vm, script)
}

#[cfg(test)]
mod tests;
