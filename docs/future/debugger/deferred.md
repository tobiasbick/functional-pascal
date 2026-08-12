# Deferred debugger scope

The following capabilities are intentionally excluded from the first debugger
release. Protocol adapters must advertise them as unsupported and must not
silently approximate them. Deferral keeps the initial engine deterministic,
read-only, and testable; it does not reject these features permanently.

Read-only expression evaluation, detached controlled calls, watches,
conditional breakpoints, exact-hit conditions, non-stopping logpoints, and
stopped-state `setVariable` and textual `setExpression` for supported mutable
values are implemented. Calls
with host I/O, nondeterminism, blocking, tasks, opaque resources, or unresolved
dynamic effects are deliberately rejected by the implemented safety policy and
are not a deferred promise. The following work remains intentionally deferred.

## Additional state and control-flow mutation

Complete-value replacement of a mutable enum, `Result`, or `Option` is
implemented through the existing mutation surfaces. Implicitly switching the
live variant through an old payload-child handle, rebinding stale handles, or
partially constructing a new payload remains deferred. Debugger initialization
of a visible mutable local or global root is implemented; descendant writes on
empty storage and skipping the later source initializer remain deferred.

Deferred:

- implicitly switching a data-carrying enum variant or a `Result`/`Option`
  wrapper through a descendant write or stale payload handle;
- assigning function values, task handles, or opaque hosted resources;
- filling uninitialized storage through field, index, or payload descendants;
- creating a missing capture cell or treating an absent parameter as
  user-initializable;
- forcing or replacing return values;
- changing the instruction pointer or restarting a frame; and
- data breakpoints or breakpoint actions that modify state.

The exact boundary and prerequisites for the uninitialized-binding slice are
recorded in
[`uninitialized-binding-assignment/consciously-deferred.md`](uninitialized-binding-assignment/consciously-deferred.md).

Bounded array insertion/removal and Unicode-scalar string character replacement
are implemented. The remaining operations need additional lvalue, source-assignment,
control-flow, lifetime, or runtime-identity semantics beyond atomic replacement
of an existing typed value.

Re-entry gate: define the chosen operation independently, including its source
visibility, mutability, ownership, lifetime, type, cleanup, and stop semantics,
then add focused atomicity and continuation tests before advertising it.

## Advanced task-debugging control and history

Deterministic launch-owned all-stop debugging of current FPAS tasks is
implemented. The following broader task facilities remain deferred:

- non-stop or parallel debug execution where one task runs while another stays
  inspectable;
- per-task continue or pause;
- debugger commands that create, cancel, restart, reprioritize, or detach a
  task, replace its result, failure, dependency, or timer, or force it runnable;
- assigning task handles through debugger mutation;
- spawn-to-child, waiter-to-dependency, scheduler-history, task-name, or
  task-group stepping shortcuts;
- persistent completed-task stacks, variables, timelines, or ancestry; and
- custom VS Code task panels, filters, scheduler visualizations, or exported
  execution traces.

Reason: all-stop current-state debugging does not define the memory consistency,
rollback, lifetime, retention, and scheduling semantics required by these
operations. The implemented debugger deliberately keeps one deterministic host
execution lane and exposes no live values while another task advances.

Re-entry gate: specify one bounded operation independently, including shared
state visibility, task ownership, identity, cancellation propagation, cleanup,
and protocol-equivalent stop behavior. Non-stop work additionally requires a
quiescence protocol that proves inspection never sees torn frames or values;
history work requires explicit retention and privacy limits.

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
