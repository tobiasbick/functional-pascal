# Debugger scope deferred beyond V1

The following capabilities are intentionally excluded from the first debugger
release. Protocol adapters must advertise them as unsupported and must not
silently approximate them. Deferral keeps the initial engine deterministic,
read-only, and testable; it does not reject these features permanently.

## Expression evaluation and watches

Deferred:

- arbitrary FPAS expressions;
- function calls from the debugger;
- watch expressions with side effects;
- hover evaluation during a stopped session.

Reason: the compiler currently compiles complete programs rather than isolated
expressions against a suspended register frame. Function calls may mutate
state, perform I/O, block, or spawn tasks.

Re-entry gate: define a read-only expression subset and prove name resolution,
type checking, resource limits, and side-effect rejection against real stopped
frames. Full evaluation requires an explicit side-effect and timeout policy.

## Variable mutation

Deferred:

- DAP `setVariable`;
- changing locals, globals, fields, or collection entries;
- forcing return values or changing the instruction pointer.

Reason: mutation must preserve FPAS immutability, runtime layouts, closure-cell
sharing, and value ownership invariants.

Re-entry gate: specify which mutable source bindings may be changed and add VM
validation that constructs only type-correct values without bypassing language
semantics.

## Conditional breakpoints and logpoints

Reason: both depend on safe expression evaluation. A textual condition must not
be treated as host code or evaluated with unbounded side effects.

Re-entry gate: complete the read-only evaluator, then specify evaluation-error,
hit-count, output-limit, and timeout behavior.

## Task and concurrent debugging

Deferred:

- programs with reachable task spawning;
- DAP threads representing FPAS tasks;
- per-task pause and stepping;
- deterministic inspection while another task runs.

Reason: the VM scheduler can execute workers concurrently and moves frame and
register state between queued task states. A debugger must not expose torn or
nondeterministic snapshots.

V1 behavior: reject the launch before program execution with an actionable,
stable diagnostic when reachable function metadata reports task spawning.

Re-entry gate: implement a deterministic debug scheduler, a stop-the-world
snapshot protocol, stable task IDs, and tests for spawn, await, timeout,
cancellation, failure propagation, and retained tasks.

## Interactive terminal, TUI, and graph applications

Deferred:

- live `Read`/`ReadLn`/`ReadKey` through a VS Code integrated terminal;
- full-screen console/TUI debugging;
- interactive graph-window event delivery while paused;
- reliable pause during a blocking host call.

Reason: DAP stdio must remain reserved for protocol traffic. Interactive
program input requires a separate debuggee process or another transport, while
TUI and graph hosts have additional event-loop and redraw constraints.

V1 may accept deterministic preloaded input in external command scripts, but
must not claim interactive terminal support.

Re-entry gate: launch a separate debuggee through the editor terminal, connect
it to the adapter through an authenticated local pipe, and prove cleanup,
cancellation, input ordering, output separation, and window-event behavior on
supported hosts.

## Attach and native executable debugging

Deferred:

- attaching to an already running `fpas` VM;
- attaching to a bundled host-native executable;
- OS-level native instruction debugging;
- remote debugging.

Reason: V1 owns execution from launch and keeps debug state in-process. Native
bundles currently contain a runner and bytecode image but no debugger transport
or rendezvous mechanism.

Re-entry gate: define process discovery, authentication, version negotiation,
source mapping, disconnect ownership, and a recoverable local transport. This
remains bytecode source debugging, not machine-code debugging.

## Advanced breakpoint forms

Deferred:

- data breakpoints;
- function breakpoints;
- exception filters beyond structured runtime failure;
- breakpoint actions that modify state.

Re-entry gate: each form needs a stable runtime identity, precise stop semantics,
bounded overhead, and protocol-equivalence tests for JSONL and DAP.

## Reverse debugging, replay, and hot reload

Deferred:

- step backwards;
- deterministic execution recording and replay;
- editing and replacing code in a suspended session;
- preserving breakpoints and frames across recompilation.

Reason: these require snapshotting or replaying VM state, hosted resources,
input, randomness, time, tasks, and external side effects. Hot reload also
changes function, layout, and register identities.

Re-entry gate: first define a deterministic hosted-runtime boundary and a
versioned snapshot/replay format. Hot reload requires explicit compatibility
rules for active frames and values.
