//! Apply parsed script events to a VM before `run()`.
//!
//! **Documentation:** [`docs/future/test-framework/scripted-input.md`](../../../docs/future/test-framework/scripted-input.md)

use super::console;
use super::graph;
use super::parse::{ScriptEvent, ScriptFile};

/// Pushes all script events into the VM input queues in order.
pub fn apply_script(vm: &mut fpas_vm::Vm, script: &ScriptFile) -> Result<(), String> {
    for (index, event) in script.events.iter().enumerate() {
        apply_one_event(vm, event, script.config.headless_graph)
            .map_err(|message| format!("script event #{index}: {message}"))?;
    }
    Ok(())
}

fn apply_one_event(
    vm: &mut fpas_vm::Vm,
    event: &ScriptEvent,
    headless_graph: bool,
) -> Result<(), String> {
    match event {
        ScriptEvent::Readln { line } => {
            vm.push_readln_input(line);
            Ok(())
        }
        ScriptEvent::ReadkeyChars { chars } => {
            vm.push_readkey_input(chars);
            Ok(())
        }
        ScriptEvent::ConsoleKey { .. }
        | ScriptEvent::ConsoleMouse { .. }
        | ScriptEvent::ConsoleResize { .. }
        | ScriptEvent::ConsolePaste { .. }
        | ScriptEvent::ConsoleFocusGained
        | ScriptEvent::ConsoleFocusLost => {
            let console_event = console::console_event_from_script(event)?;
            vm.push_console_event(console_event);
            Ok(())
        }
        ScriptEvent::GraphKey { .. }
        | ScriptEvent::GraphMouse { .. }
        | ScriptEvent::GraphWheel { .. } => {
            if !headless_graph {
                return Err(
                    "graph events require `[config] headless_graph = true`.\n  help: Add `[config] headless_graph = true` to the sidecar script."
                        .to_string(),
                );
            }
            let graph_event = graph::graph_event_from_script(event)?;
            vm.push_graph_event(graph_event);
            Ok(())
        }
    }
}
