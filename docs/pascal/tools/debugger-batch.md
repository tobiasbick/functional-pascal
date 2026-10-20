# One-shot batch debugging

`fpas debug [target]` runs once to the first requested breakpoint, first runtime
failure, or normal completion. It returns one UTF-8 JSON object followed by a
newline on stdout, then exits. This is the default debugger mode for agents.
Use `--protocol dap` or `--protocol jsonl` for an existing long-lived session.

```sh
fpas debug app.fpasprj --break src/main.fpas:42 --print Total --print 'Items[Selected]'
fpas debug app.fpasprj --script input.toml --timeout 5 -- input.txt
fpas debug app.fpascp --source-root . --print Total
```

Targets and discovery match the [source debugger](debugger.md): program source,
program project, workspace with one program, or verified compiled image. Omitting
the target discovers a workspace/project in the current directory. Direct images
require `--source-root`; every embedded source identity is checked before launch.
Stale, missing or escaping sources are errors.
Use the `Debug source root` printed by `fpas build`. Build and project debugging
share a [common source root](../program-structure/compiled-programs.md#source-roots-for-debugging)
covering the program and resolved dependencies, including sibling projects.

## Request

| Option | Meaning |
| --- | --- |
| `--break path:line[:column]` | Repeatable source location, at most 64; coordinates start at 1. |
| `--print expression` | One expression per occurrence, at most 64; never comma-split. |
| `--trace selector` | Scalar global or exact `Routine::Name` local/parameter/capture, with stored fields/literal array positions; repeat, at most 64. |
| `--trace-limit count` | Complete trace event capacity, default 1,000, ceiling 10,000; requires `--trace`. |
| `--schedule-seed N` | Explicit unsigned 64-bit seed for fair task rounds, virtual task time and controlled effects; text-line input only. |
| `--task id` | Inspection task; defaults to the stopping task. |
| `--frame depth` | Inspection stack depth; defaults to 0, the active frame. |
| `--script path` | Explicit runner-compatible TOML input script. |
| `--timeout seconds` | Positive whole-invocation deadline; default 30 seconds. |
| `--instruction-limit count` | Positive dispatched-instruction bound; default 100,000,000. |
| `--output-limit bytes` | Positive combined output bound; default and ceiling 1,048,576 bytes. |
| `--source-root dir` | Common source root printed by `fpas build`; required for a compiled image. |
| `--std-lib dir` | Replace the complete source standard library. |
| `-- arguments...` | Program arguments, available through `Std.Args`. |

These batch-specific inspection/input options cannot be combined with an explicit
protocol. `--commands` still requires `--protocol jsonl`. Batch has no `--batch`,
`--report`, `--until` or text-output switch. Help and version requests retain ordinary CLI output.

Breakpoint suffixes are parsed from the right, including Windows drive paths.
Source paths must identify exactly one loaded source. Equal requests are
deduplicated in request order. All breakpoints must bind before any program
statement runs. The existing source binder chooses the executable statement
position and reports its resolved location. A breakpoint stops **before** its
statement. The first stop ends the invocation; batch never resumes automatically.

Input and expression syntax are validated before bytecode runs. Expressions may
read literals, visible bindings, stored fields and indexes, and use scalar unary
or binary operators. They cannot invoke functions, methods, getters, constructors,
default expressions, or assignments. An unresolved dotted name never constructs
an enum value. All expressions inspect the same frozen state; an individual
lookup/index/type error preserves other successful results.

The debugger normally uses its stable all-stop task scheduler with real task
time. Opt into [seeded scheduling](debugger-scheduling.md) with
`--schedule-seed N` to vary fair task rounds and use virtual time under controlled
effects. Neither mode reproduces the normal concurrent worker pool or provides
external-event replay.
For a failure from an ordinary concurrent run use
[`fpas run --failure-dump`](failure-dump.md).

## Input and output

Scripts use the [test runner's input format](../std/testing/test.md):

```toml
[[event]]
type = "readln"
line = "answer"

[[event]]
type = "readkey_chars"
chars = "ab"
```

All events are queued before the one resume. Line and key queues each preserve
their order; there is no timed ordering between the two queues. The script file
and total queued input each have a 1 MiB bound. No implicit sidecar is loaded.
No host stdin or host terminal is read. The batch runner signals EOF after queuing
the script. Once pending line input is consumed, `ReadLn` and `ReadText` return
`None`; key polling retains the headless console's existing behavior.
There is no new mouse, resize or timed-replay format.

Program console text and terminal rendering bytes appear in `output` with
`stream: "stdout"`; ANSI/control bytes are JSON-escaped, not interpreted by the
host terminal. Native child stdout and stderr use their respective stream names.
Hosted console writes and native child stdout share the same pipe and preserve
write order. Separate stdout and stderr pipes are ordered by capture, without
a cross-stream clock. Each record has `text` and `truncated`.
Complete UTF-8 characters survive chunk boundaries; malformed or EOF-incomplete
native byte sequences use the Unicode replacement character. Combined captured
text is bounded. Exceeding its bound ends the invocation with an explicit limit.
The JSON writer adds no raw program output to stdout or stderr.

## Result schema, version 1

Every ordinary batch result, including invalid options and build/load errors,
contains these fields:

| Field | Content |
| --- | --- |
| `schema_version` | Integer `1`. |
| `outcome` | `breakpoint`, `runtime_error`, `completed`, `limit`, or `error`. |
| `exit_code` | The process exit code below. |
| `stop` | Stopping task, reason, source location and bound requests, or null. |
| `diagnostics` | Structured FP diagnostics or batch `code`/`stage`/`message` records. |
| `sources` | Source IDs and portable paths; no source text. |
| `globals` | Globals captured once in declaration order. |
| `tasks` | Actual task IDs/states and bounded logical stack frames. |
| `expressions` | Requests in order with selected context, typed value or error. |
| `output` | Ordered captured text records. |
| `trace` | Bound watches and ordered typed initialization/change history, with explicit coverage and counts. |
| `schedule` | Requested scheduling mode/seed, observed counters and explicit controlled/rejected/interrupted/unavailable coverage. |
| `capture` | `incomplete`, omission `reasons`, and known task/global totals. |
| `limits` | Effective execution, output, report and snapshot bounds. |

Frames identify depth, function, source location, reason and lexical bindings.
Bindings identify name, parameter/local/capture kind, scope ID and value. These
are detached data, with no session-scoped variable references. Globals are not
repeated in every frame. The stopping task has capture priority, followed by an
explicitly selected peer and then the remaining task IDs.

Typed values share the [failure-report representation](failure-dump.md): `type`,
`kind`, `status`, `scalar`, `summary`, `children`, `total_children`, `reason`.
FP diagnostic text is bounded to 4,096 scalars with an explicit `truncated` flag;
complete oversized fix-its and diagnostic tails may be omitted.
Integers use decimal strings to preserve their full range. Named/indexed children
retain their order; dictionaries retain ordered key/value children. Truncation,
cycles, uninitialized and unavailable values have explicit status/reason markers.
An ordinary `Error` or `None` value is data, not a runtime failure.

At normal completion, available globals remain inspectable. No local frame is
invented after return. Default expressions can read globals; an expired local
fails independently. An explicit `--task` or `--frame` after completion is an
invalid inspection context and returns exit 3; it never silently falls back.

| Bound | Ceiling |
| --- | --- |
| Tasks | 64 |
| Frames per task / in total | 32 / 256 |
| Bindings in total | 4,096 |
| Value depth / immediate children | 8 / 64 |
| String content | 4,096 Unicode scalars |
| Complete escaped JSON, including newline | 8 MiB |

Automatic snapshot truncation alone does not fail a fulfilled request. Requested
expression/context failures do. When the escaped document needs further clipping,
omissions are marked; primary outcome/diagnostics and requested values receive
priority over optional state. If a requested value must be omitted, its expression
receives an error. Runtime failure remains exit 2 despite an optional inspection
failure; an independent execution/worker limit takes priority and returns 3.

## Exit codes and lifecycle

| Exit | Meaning |
| --- | --- |
| 0 | Requested stop or ordinary completion fulfilled. |
| 1 | Invalid request, script, parsing/build, load, or breakpoint binding. |
| 2 | Runtime failure. |
| 3 | Execution/output limit, worker/debugger failure, missed breakpoint, or failed requested inspection. |

A program that completes without reaching any explicitly requested breakpoint
has outcome `completed`, diagnostic `breakpoint_not_hit`, and exit 3. A failed
inspection preserves the actual execution outcome.

The CLI parent owns an isolated worker and the invocation deadline. The deadline
starts before target/script preparation and includes execution and capture.
On timeout or output overflow, the parent requests a cooperative stop, allows at
most 250 ms grace, then terminates the worker's process tree and reaps it. The
same Unix session / Windows process-tree owner serves the test runner. Unix
cleanup includes nested process groups created by `Std.Proc`, including children
reparented after the worker dies. Descendants must remain in the worker's session.
Private request/result/cancellation files are removed after the invocation.

After forced termination, only complete acknowledged output records and an
atomically published snapshot can be retained. Missing state is explicitly
unavailable; it is never filled with an earlier invented frame. Cancellation
cannot roll back filesystem/network side effects or guarantee native cleanup.
Uninterruptible OS operations and a blocked final stdout write are outside the
wall-clock return guarantee. External host kills, OOM and crashes cannot promise
a final JSON result. Stderr is otherwise empty except a final publication error.

Existing JSONL and DAP protocols, documentation and tests remain supported.

## Tracepoints

`fpas debug app.fpasprj --trace Total --trace State.Selected` keeps a bounded
history while running to the existing first stop/end. It records initialization
and consecutive unequal successful stores, including `var` and callback writes,
with typed old/new values and the actual writing instruction/task. Mutable globals
belong to the main task. Use `--trace 'Routine::Name'` for scalar locals and
parameters across calls/tasks or active callback captures, optionally followed
by stored fields such as
`Routine::State.Selected` or literal indexes such as `Routine::Items[0]`, with
activation and shared declaration-instance identities. Missing array positions
are known unavailable values, not incomplete history.
Concrete source projections preserve applied generic field types and nominal
array element names through references and captures; unresolved generic types
remain unsupported.
Local observation ends at lexical scope/call exit. Capture selectors such as
`MakeCounter.Next::Total` initialize at each active call, observe its live aliases
and end at that call's exit; dormant changes are outside their coverage.
Anonymous callbacks use source-ordered names such as `MakeCounter.$closure_1`,
stable for identical compilation inputs; use the full name from the stack.
Tracepoints do not create additional stops. Trace limits end execution explicitly
with exit 3.
The [tracepoint contract](debugger-tracepoints.md) defines selectors, the additive
`trace` JSON object, exact comparison, event/byte bounds and missing-history rules.
