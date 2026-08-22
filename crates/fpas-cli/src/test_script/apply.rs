//! Apply parsed script events to a VM before `run()`.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use super::parse::{ScriptEvent, ScriptFile};

/// Pushes all script events into the VM input queues in order.
pub fn apply_script(vm: &mut fpas_vm::Vm, script: &ScriptFile) {
    for event in &script.events {
        apply_one_event(vm, event);
    }
}

fn apply_one_event(vm: &mut fpas_vm::Vm, event: &ScriptEvent) {
    match event {
        ScriptEvent::Readln { line } => {
            vm.push_readln_input(line);
        }
        ScriptEvent::ReadkeyChars { chars } => {
            vm.push_readkey_input(chars);
        }
    }
}
