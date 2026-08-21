# Functional Pascal

A function-first language that compiles `.fpas` sources to bytecode and runs them on a managed virtual machine.

## Language

**Debug engine**:
Protocol-neutral debugger execution for one launch-owned target. Adapters map wire spelling onto it; they do not own stop policy, evaluation, or live-image rules.
_Avoid_: JSONL core, debug core, debug server (when meaning this module)

**Prepared debug target**:
A verified program image plus portable sources and execution limits, ready to launch in a debug engine.

**Debug session**:
VM-owned execution of one prepared debug target under debugger control. Launch-owned: the debugger constructs an in-process VM and does not attach to a running process.
_Avoid_: debuggee process, attached process
