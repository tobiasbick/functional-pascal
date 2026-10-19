# Batch tracepoints

`fpas debug --trace` records initialization and consecutive changes to selected
scalar storage while running to the existing batch stop/end or a resource limit.
It returns the usual single JSON result with a `trace` object, without opening a
debugger session or writing a trace file.

```sh
fpas debug app.fpasprj --trace Total --trace State.Selected
fpas debug app.fpasprj --trace Total --trace-limit 200 --script input.toml
fpas debug app.fpasprj --trace Total --break src/main.fpas:42 --print Total
fpas debug app.fpasprj --trace 'Calculate::Total' --trace 'Calculate::Count'
```

Targets, arguments, scripts, source verification, instruction/output bounds and
lifecycle follow [batch debugging](debugger-batch.md). Trace flags are batch-only;
explicit JSONL/DAP sessions retain their own data-breakpoint/logpoint behavior.

## Selecting storage

Repeat `--trace <selector>` up to 64 times, with 1–4,096 UTF-8 bytes per selector.
A global selector is a dotted identifier path rooted at a mutable executable
global. It may continue through at most 16 ordinary stored record fields, ending
at a scalar. Examples are `Total`, `State.Selected` and `My.Unit.Counter`.

A local selector uses exact executable `Routine::Name`, such as
`Calculate::Total`. It selects one visible scalar source local or parameter;
`const` locals are eligible too. A `var` parameter follows its scalar referent.
The routine name is the full executable metadata name, including any unit,
method or specialization components. There is no import alias, suffix search or
function-value alias resolution. Duplicate function or lexical binding matches
are rejected; give shadowed declarations unique names when tracing them. Locals
require exact declaration-store metadata; unsupported implicit loop bindings
are rejected during setup. Local fields, indexes and capture bindings cannot be
selected. Selector resolution executes no FPAS code.

Names match ASCII case-insensitively. Equivalent identities are deduplicated in
request order, retaining the first spelling. Unit roots use their full executable
metadata unit name, not an import alias or unqualified suffix search. The whole
path must identify exactly one eligible binding. Missing, ambiguous, immutable
global or unsupported targets fail the whole request before any statement or
initializer runs. Types and field slots come from metadata, without initialized values.
Private retained globals/fields remain inspectable by the debugger.

Supported leaves are integer, real, Boolean, string, integer subrange, distinct
scalar and enums with no payload-bearing variants. Declared nominal scalar
names are retained in typed values; transparent aliases preserve their underlying
type. Whole aggregates, Option/Result values, capture bindings,
task/channel/callable/resource values, indexes, getters, method calls and
arbitrary expressions are unsupported.

Mutable program/unit globals belong to the main task, so these events normally
report task 0. Local selectors also observe worker-task locals and parameters.
Tracing does not make globals shared. The debugger's deterministic scheduler does not reproduce an ordinary
concurrent run; [failure reports](failure-dump.md) retain that run's failure state.

## Observed changes

The first committed value produces `initialize`, with an old typed value whose
status is `uninitialized`. Later unequal commits produce `change`. Equality uses
actual scalars, including text beyond the displayed prefix, rather than summaries.
Signed real zeros compare equal. Global NaN values in a verified external image
compare unequal; local observations require finite reals and end explicitly with
`trace_value_limit` for nonfinite external values. Ordinary FPAS numeric operations
retain their finite-value validation. Repeated equal writes produce no event.

Save this as `counter.fpas`; `fpas debug counter.fpas --trace Total` reports new
values `1`, `2`, `1`, including the initialization:

```pascal id=trace-counter run
program TraceCounter;

var Total: integer := 1;

begin
  Total := 2;
  Total := 2;
  Total := 1;
end.
```

Each successful store is observed, including `var` parameter writes and callbacks.
Replacing a global record can change several watched fields; unrelated field
writes create no false events. Failed stores produce no successful-change event.
Old/new values are detached, without retaining entire live aggregates or cells.

Parameters initialize on call admission, before the callee's first instruction.
Locals initialize at their declaration store. Each call gets a distinct
`activation_id`; each selected declaration lifetime gets an `instance_id`.
These decimal-string identities distinguish recursion, tail calls, repeated
declarations in loops, tasks and reused callback workers. Repeating a declaration
produces a new `initialize` even when its value is equal to the previous instance.
A live local follows writes through scalar `var` aliases and nested callbacks.
Observation ends when its lexical scope or call ends; later writes to an escaped
capture are outside that local's history. There are no synthetic scope-exit or
return-value changes. Detached retained events survive call completion.

Save this as `local-counter.fpas`. With `--trace 'Calculate::Total'`, the new
values are `1`, `2`, `4`, `5`; each call starts a new activation and instance:

```pascal id=trace-local-counter run
program TraceLocalCounter;

function Calculate(Count: integer): integer;
begin
  var Total: integer := Count;
  Total := Total + 1;
  return Total;
end function;

begin
  const First := Calculate(1);
  const Second := Calculate(4);
  if (First <> 2) or (Second <> 5) then
    panic('unexpected counter result');
  end if;
end.
```

Events follow execution order; watches changed by one commit follow request
order. Attribution uses the actual writing instruction/function/task, including
callbacks, rather than the next statement. Unavailable source information is
null. A breakpoint stops before its statement, so that statement's future writes
are absent. Final `--task`/`--frame` options do not filter trace collection.

## JSON

Every batch result has this additive schema-version-1 object, including errors
and invocations without tracing:

| Field | Meaning |
| --- | --- |
| `trace.requested` | Ordered selectors; successful binding deduplicates equivalent requests. Setup failures retain available raw requests. |
| `trace.watches` | Zero-based `id`, first `request`, canonical `name`, `identity` and declared `type`. Global identity has `global_slot`, `field_slots`; local identity has `function_id`, `binding_id`, `scope_id`, `kind` (`local` or `parameter`). |
| `trace.events` | Earliest complete event prefix in execution order. |
| `trace.complete` | Every qualifying event through the actual final stop/end is retained. |
| `trace.reasons` | Explicit missing-history or value-presentation truncation reasons. |
| `trace.event_count` | Number of retained complete events. |
| `trace.total_count` | Exact qualifying count when known, otherwise null. |

Each event contains one-based `sequence`, `watch_id`, `kind` (`initialize` or
`change`), actual `task_id`, `function`, `location`, `activation_id`, `instance_id`,
`old_value` and `new_value`. Global events use null activation/instance identities.
Local identities describe the watched declaration; function/location/task describe
the actual admission or writing instruction, including writes by a callee or callback.
Delayed results from callbacks and suspending operations such as `Tasks.Wait`
are attributed to their original call statement and appear before a breakpoint
on the following statement. A suspended operation keeps its declaring scope live
through the final callback/result commit, even at the end of a lexical block.
Location uses portable `source`, one-based `line` and `column`. Values share the
[D1/D3 representation](failure-dump.md), including decimal strings for integers
and explicit truncation/uninitialized markers. No live inspection handles escape.

Without tracing the lists are empty, counts zero and coverage complete. A valid
watch never initialized before the final stop can have no events. Complete
coverage describes the executed prefix, not future program paths. Binding
failure has no partial watch set and names `binding_failed`. Other setup failures
or unpublished worker state use `state_unavailable`, never a fabricated successful
empty trace.

## Bounds and outcomes

`--trace-limit <count>` is one positive event capacity across all watches:
default 1,000, maximum 10,000. It requires at least one `--trace`. Exactly filling
it succeeds if no further qualifying change occurs. The next event that cannot
fit ends execution at the next safe boundary with `outcome: limit`, exit 3 and
`trace_event_limit`. The triggering store may already have committed.

Other trace bounds are 65,536 UTF-8 bytes per exact scalar string value, 8 MiB
of comparison strings retained for one global commit, 16 MiB of cumulative string-comparison
input and 2 MiB of complete escaped trace control/event JSON. Comparison bytes
are charged before scanning, including equal stores. Exceeding these bounds ends
execution with `trace_value_limit`, `trace_work_limit` or `trace_byte_limit`.
Unknown equality is never treated as equal; there is no sampling or ring buffer.

Local observation additionally allows at most 1,024 live selected declaration
instances and 8 MiB of their retained comparison strings, conservatively counting
each instance even when strings share storage. Scope/call exit releases these
values. Exceeding either bound ends with `trace_value_limit` and unknown total
coverage. `limits.trace_live_instances` and
`limits.trace_local_retained_value_bytes` expose these ceilings. Unchanged local
strings with identical immutable storage need no scan or comparison-work charge.

Value presentation retains at most 4,096 string scalars. Presentation truncation
alone adds `value_truncated` and keeps complete event coverage. Missing events
make `trace.complete` false and propagate incomplete coverage to top-level
`capture`. Counts stay unknown when comparison or remaining affected watches
could not be inspected.

The whole batch result still has its 8 MiB escaped-byte ceiling, including the
newline. Optional automatic state is reduced before requested history; further
clipping removes whole tail events and marks missing history. `--print` results
share the budget. Lost requested history changes otherwise successful exit 0 to
3. Result clipping preserves runtime failure and exit 2; an independent execution
limit takes exit-3 priority.

The existing whole-invocation deadline, cancellation and process-tree cleanup
still apply. Forced termination can lose all unpublished trace state, with
explicit unavailable history and unknown totals. There is no incremental spool,
replay or crash-recovery format. Tracing can be combined with
[seeded scheduling](debugger-scheduling.md) for controlled task-order exploration.
