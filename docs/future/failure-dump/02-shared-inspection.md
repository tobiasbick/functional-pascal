# 02: shared read-only inspection

Depends on [01](01-contract.md). Progress lives in the [overview](README.md).

## Goal and deliverable

Extract the common read-only frame, binding, location and value inspection used
by the existing debugger, making it usable at a normal worker's fault boundary.
Keep debugger stop generations, mutation targets and variable handles in the
debugger. Avoid creating a second type renderer or lexical-visibility resolver.

## Implementation boundaries

Inspect [frame capture](../../../crates/fpas-vm/src/vm/debug/inspection/capture.rs),
[typed bindings](../../../crates/fpas-vm/src/vm/debug/inspection/typed_bindings.rs),
[value rendering](../../../crates/fpas-vm/src/vm/debug/inspection/render.rs),
[worker state](../../../crates/fpas-vm/src/vm/worker.rs) and
[register initialization](../../../crates/fpas-vm/src/vm/register_stack.rs).

Introduce focused modules under `crates/fpas-vm/src/vm/inspection/` for the
actual common operations, with debugger adapters staying in their existing
directory. Confirm the exact move/create paths before implementing. Avoid a
new crate or public interface that exists only for tests. VM-facing inspection
must read worker-local globals, initialized registers, references, callback
continuations and capture cells correctly under the contract's budgets.

The failure report must own stable data. Sharing a live mutable cell or a
debugger handle into a later serialization phase is not a snapshot. Capture
uses nonblocking access to shared state and never invokes FPAS code.

## Acceptance criteria

- Parameters, shadowed locals, captures, `var` parameters and globals resolve
  to the actual visible values; unavailable and uninitialized values differ.
- Caller positions and active instruction positions are correct in normal
  execution, including failure at a hosted callback boundary.
- Aggregates, cycles, missing metadata and each applicable value bound follow
  the version-1 value contract without unbounded intermediate allocation.
- Ordinary debugger stack/scopes/variables and DAP/JSONL behavior retain their
  current semantics. Mutable debugger features continue through their adapters.
- No successful-instruction tracing or failure-capture activation is introduced
  as a side effect of the refactor.

## Verification to record on delivery

Use the existing VM debugger inspection and aggregate tests as their primary
owners, adding cases for normal workers only where debugger tests cannot
exercise the different register/global/callback lifecycle. Run the affected
VM/debugger tests, formatting, build and required workspace checks. Record
actual commands, results and any limitations here when implementing this step.
