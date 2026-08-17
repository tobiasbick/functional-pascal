# Umbrella architecture

## One engine, multiple adapters

Debugger behavior is owned by the Rust runtime and debug session:

```text
compiler metadata and portable artifacts
                  |
                  v
crates/fpas-vm/src/vm/debug/       shared execution and policy
                  |
          +-------+--------+
          |                |
          v                v
      JSONL server       DAP server
          |                |
          |                v
          |        editors/vscode debugger UI
          v
 external tools and LLM agents
```

JSONL, DAP, and VS Code may map transport concepts, but they must not own a
second implementation of execution, mutation, task control, breakpoint, or
identity policy.

## Shared invariants

### Runtime identity

- Function IDs, task IDs, frame IDs, binding IDs, cell allocations, and stop
  generations are identities. Display names and rendered values are not.
- A protocol reference is valid only for the stop generation and task that
  created it unless a package explicitly proves a longer lifetime.
- Task-bound functions retain an explicit runtime owner and never become
  transferable through detachment, adapter conversion, or display parsing.
- Portable artifacts contain reconstructible metadata, never process-local
  pointers or runtime task owners.

### Execution ownership

- `dispatch_one` remains the controlled VM execution boundary.
- The initial task model is deterministic launch-owned all-stop debugging.
- Per-task or non-stop operations require the quiescence contract in `UMB-40`.
- Pause inside hosted work is cooperative at VM instruction boundaries. An
  in-progress host intrinsic cannot be interrupted; pause is observed after it
  returns. Unsafe thread termination is forbidden.
- Attach and remote sessions remain launch-owned. Connecting to an already
  running `fpas run` process is rejected. Native OS debugging of the host
  process is not a second semantic engine; it is rejected.

### Prepare, validate, commit

Every state-changing operation follows one bounded transaction:

1. Resolve stable identities from the current stop.
2. Validate type, ownership, lifetime, limits, and effect policy.
3. Prepare all replacement or control state without mutating the debuggee.
4. Commit once.
5. Refresh affected snapshots and invalidate old handles once.
6. Return equivalent machine results through JSONL and DAP.

Failures before commit preserve execution state, values, cell identities,
frames, tasks, scheduler state, and inspection generation.

### Effects and bounds

- Calls with unresolved or forbidden effects remain closed by default.
- Every new history, transport, snapshot, or breakpoint facility has explicit
  memory, count, depth, time, and output bounds.
- Cancellation and timeout behavior are part of the package contract, not
  optional hardening after implementation.
- Recording and replay must define how nondeterministic host events are
  captured or rejected before any history is retained.

### Protocol parity

- External and LLM automation uses deterministic JSONL records with stable
  error kinds and actionable hints.
- VS Code uses standard DAP requests and capabilities where they describe the
  feature completely. A custom request is added only when DAP has no adequate
  representation.
- A custom DAP request must have an equivalent JSONL operation and shared VM
  test coverage before editor code is added.
- Capability negotiation controls adapter presentation; it does not weaken VM
  validation.

### Language boundary

The umbrella may change debugger/runtime/tooling behavior and portable debug
metadata. It does not change FPAS syntax, static semantics, runtime language
semantics, or the language specification without separate explicit agreement.

## Ownership by layer

| Concern | Primary ownership |
|---|---|
| Debug metadata emission | compiler and bytecode metadata crates |
| Metadata preservation | unit, linker, and program artifact crates |
| Stops, inspection, mutation, calls, tasks | `fpas-vm` debug session |
| Machine protocol | `fpas-debug` JSONL server |
| Editor protocol | `fpas-debug` DAP server |
| Editor commands and UX | `editors/vscode` |
| Launch/build/discovery | CLI and project/build crates |
| Current behavior documentation | `docs/pascal/tools/` |

New work must stay in the narrowest existing ownership boundary. Large or
mixed modules are split by concern before adding more behavior.

## Shared foundation before advanced packages

The following proofs are prerequisites rather than adapter-specific features:

- stable runtime and portable identities;
- exact stop-generation ownership;
- scheduler quiescence and waiter effects;
- debuggee I/O separation from protocol I/O;
- bounded snapshots and event logs;
- version and source compatibility for attach, replay, and hot reload.

Packages may reuse a completed proof. They may not introduce a local substitute
that creates divergent behavior.

