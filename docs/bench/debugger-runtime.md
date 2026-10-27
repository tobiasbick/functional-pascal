# Debugger runtime

The debugger schedules one instruction at a time on a single execution lane.
Source, function and data breakpoints, tracing, pause and execution limits are
checked at instruction boundaries. Batch, DAP and JSONL sessions use this runtime.
See the [source debugger](../pascal/tools/debugger.md) for the supported commands
and protocol contracts.

## Execution ownership and roots

Execution access spans at most 256 scheduler iterations. The runtime publishes
retained task roots before releasing that access, returning a stop or error, or
waiting for input. Pending maintenance can publish earlier. Tasks spawned during
a batch use their own execution ownership boundary until the next batch begins.
The supervision wrapper delegates root publication to its enclosing execution
owner.

The ordinary `fpas run` path uses its own scheduling and root-publication policy.

## Breakpoint lookup

Source and function breakpoints share a session-local index of executable
instruction addresses. Each address retains its sorted logical breakpoint IDs.
The runtime looks up the next sequence-point address in this index; matching
source, function and data breakpoint IDs are reported together in ascending order.

Setting, clearing or replacing code breakpoints rebuilds the index. Reload and
rollback prepare it from the candidate image's bound locations before committing
the image. Unverified breakpoints retain their logical identities without an
address entry. Conditions, hit counts, logpoints and breakpoint assignments use
the resulting logical IDs in the protocol layer.

## Terminal input and readiness

The debugger owns a queue-only terminal. A positive `Std.Console.ReadEventTimeout`
suspends the calling task until input arrives or its deadline expires.
Zero-timeout reads return immediately. Input consumption and result delivery run
on the VM thread. Other runnable tasks can execute while the reader waits.

Readiness checks use an input revision and the task's deadline. Until input arrives
or the deadline expires, the debugger checks the revision without locking the
input queue or entering the full suspension poll. Pure sleeps use their deadline
to determine readiness. Scope cancellation is checked at each scheduling boundary.
New input, expiry and cancellation use the guarded poll and cleanup path.

Each reader snapshots its input revision while holding the queue lock, so input
arriving between an empty read and suspension is visible on a later readiness
check. Queued input takes precedence when a read's deadline expires.

Other suspension kinds use their own readiness checks. Inspecting tasks, stacks
and variables does not dispatch tasks or consume queued terminal events.

## Profiling and timing

Use [`fpas debug --profile`](../pascal/tools/profiler.md#debug-targets) to inspect
routine and line samples together with scheduling, dispatch, root-publication,
waiting and inspection times for the debug target. Profiling an FPAS controller
with `fpas run --profile` reports the controller's work; it does not attribute the
debugger child's phases.

Profiling adds timing overhead. For end-to-end latency measurements, use
unprofiled release executables, identical compiled images and source roots, fixed
input and terminal dimensions, and checked program results. Define which startup,
protocol, inspection, rendering and shutdown work belongs inside the timer.
Keep builds, tests and other benchmark jobs outside the measurement interval.

Use the [benchmark workflow](README.md) for controlled comparisons and ordinary
execution controls. Keep raw profiles, controller output and temporary benchmark
data under `.temp-data/`.
