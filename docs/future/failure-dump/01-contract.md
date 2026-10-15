# 01: failure-dump contract

Recorded on 2026-10-15 against commit `853ed527` as the initial design contract.
The option and report are now implemented in [step 05](05-cli-and-json-report.md);
the implemented user contract lives in [runtime failure reports](../../pascal/tools/failure-dump.md).
Current progress lives in the [overview](README.md).

## Invocation and publication

One opt-in option, accepted only by `run` before the argument separator:

```text
fpas run app.fpasprj --failure-dump .temp-data/failure.json
fpas run --diagnostics json --failure-dump .temp-data/failure.json app.fpascp -- input.txt
```

It applies to every existing run input: source, project, workspace, compiled
image, and project/workspace discovery. Relative output paths resolve against
the invocation directory. No manifest setting, environment-variable alias,
automatic default filename, or second report mode is introduced. Arguments
after `--` remain program arguments. The option is not added to `test`, `debug`,
`check`, `build`, or bundled applications in D1.

The destination must be a new file. An empty path, repeated option, missing
value, directory destination, or existing file is rejected before execution
with the existing CLI argument diagnostic, FP4137. Do not silently replace a
previous report or accept `-` as a stdout destination. Missing parent
directories are created only when publishing a failure report.

No report or staging file is created on success, compilation/load failure,
or argument failure. On a qualifying runtime failure, write one UTF-8 JSON
document with a trailing newline through a sibling staging file. Publish it
atomically without replacing an existing destination, including a file created
by a competing process after validation. Remove owned staging files after a
failed publication. Never expose partially written JSON at the final path.

Program stdout and the existing text/JSON stderr diagnostics keep their current
contracts. Successful publication adds no new stdout or stderr record. A write
failure keeps the original runtime diagnostic and adds FP4139 in the requested
diagnostic format. The runtime-failure exit code remains **2**, including when
report publication fails; successful execution remains **0** and pre-execution
errors remain **1**. A consumer identifies the new report by the requested fresh
path and must inspect completeness fields inside it.

## Trigger and causal identity

Publish only for a runtime diagnostic that makes the top-level run fail.
Ordinary `Error(...)` and `None` are values. A task failure handled through the
existing supervision or application-host mechanisms does not publish a report.
Normal task cancellation during successful shutdown does not publish one either;
a cancellation diagnostic that actually escapes the run does qualify.

Capture a failure candidate's original frames and values before its worker
unwinds, retries, releases registers, or is disposed of. Retain that capture
only while the associated failure can still propagate. Retried, recovered or
released failures must release their captures, without keeping a run history.

Use an explicit runtime failure identity through task waits, scope/group
propagation, and diagnostic clones. The selected report error must be exactly
the failure selected by existing VM semantics; a diagnostic produced by cleanup
must not overwrite an earlier primary cause. Do not match failures by rendered
message, source position, or numeric diagnostic code alone. Preserve other
attached failures as structured `related_errors`, in existing attachment order.
They share the report's task/source catalog instead of opening further files.
Related failures describe those known at the observation cut, not failures
that have yet to occur during later cleanup.
Copying a message into a newly raised panic creates a new failure identity.

Nested hosted callbacks retain their FPAS frames and their calling FPAS frames
where that continuation exists. Native host internals are not invented as FPAS
frames. A runtime diagnostic raised without a live frame still produces a
report with the available diagnostic and an explicit unavailable capture.
Unrecoverable host aborts, out-of-memory termination, process kills and native
crashes cannot promise a report. A Rust failure already converted into a normal
VM diagnostic follows the same available-data rules.

## Task observations and cleanup

The cause snapshot records the original failing task at its fault boundary.
The task catalog records a later, explicitly cooperative observation during
terminal failure handling. It is not an atomic snapshot of all tasks at the
original fault instant. Preserve this distinction in `capture.consistency` and
each task's `observation` field.

The catalog contains the main task (id `0`), all cause tasks, every task live
at the catalog cut, and completed/cancelled tasks whose outcomes are still
retained at that cut. It does not archive already released task history.
Admission and handoff races must place each relevant task exactly once in the
catalog. Never dispatch a queued task merely to inspect it.

Running peers acknowledge a capture request at an existing bounded execution
boundary. Queued, sleeping and waiting tasks are read from their saved state.
A peer in a blocking host call need not acknowledge before the collection
deadline: retain its catalog identity and known state, and mark its frames
unavailable. Never read another running worker's registers concurrently or
wait indefinitely for its locks. State names are `runnable`, `running`,
`waiting`, `sleeping`, `completed`, `failed`, and `cancelled`; `running` may
include execution inside a host call. Unknown wait details remain unavailable.

The cause is retained before its destructive cleanup. Peer collection and
report publication must be possible before remaining shutdown joins can block
once the escaping primary failure is determined. If existing failure selection
still depends on unfinished owned work, retain candidates without guessing a
final primary or bypassing that ownership barrier. D1 does not detect deadlocks.
Earlier semantic scope cleanup may already have occurred before a failure is
known to escape; the peer catalog must not pretend to precede that cleanup.
Separate collecting immutable data from serializing/publishing it. Release
capture coordination and locks before file I/O, and let ordinary cancellation
and ownership cleanup proceed even when capture or publication fails. Preserve
the existing cleanup and termination guarantees, including waits for host work.

## JSON version 1

One object, no JSONL stream. Object key order is not significant. Required keys
are always present; unavailable optional values use JSON `null`. Readers must
check `schema_version`. Paths use portable source identities from the compiled
image, with `/` separators. Unknown locations use `null`; never substitute a
host path, working directory or guessed source line. `source_id` disambiguates
sources when only a portable basename is available. Do not read sources from
disk to manufacture missing debug information.

| Top-level field | Meaning |
| --- | --- |
| `schema_version` | Integer `1` |
| `error` | Primary error object |
| `related_errors` | Other attached error objects, or `[]` |
| `sources` | Source objects: `id`, portable `path` (or `null`) |
| `capture` | Completeness and bounds described below |
| `globals` | Bounded array of binding objects captured once for the report |
| `tasks` | Bounded array of task objects |

An error object contains `code` (`FPnnnn`), `message`, `hint` (string or `null`),
`task_id` (integer or `null` when the VM has no owning task), and `location`.
A location contains `source_id`, `line` and `column`; coordinates are one-based.
Missing location is `null`, rather than zero coordinates.

A task object contains `id`, `name`, `state`, `observation` (`fault` or
`failure_handling`), `inspection` (`available`, `partial`, `unavailable`, or
`not_applicable`), `unavailable_reason` (string or `null`), `total_frames`
(integer or `null` when unknown), and `frames`.
The primary cause uses its retained fault frames even if its worker has since
terminated. Other tasks use their catalog-time state. Completed/cancelled peers
have no fabricated stack and use `not_applicable`. A failed peer can expose
its retained failure frames with `observation: "fault"`; peers captured live
use `failure_handling`. Task ids are runtime identities, not host threads.

A frame contains `depth` (active frame is zero), `function`, `location`,
`bindings` and `truncated`. It represents an actual FPAS call, with callers
ordered outwards. Parameters, visible locals and captures are bindings with
`name`, `kind` (`parameter`, `local`, `capture`, `global`), lexical `scope_id`
(or `null` for globals), and `value`. Keep scope identities for shadowed names;
exclude compiler temporaries and out-of-scope bindings. An uninitialized binding
is explicitly uninitialized, rather than a stale register value. `var`
parameters show the current referent value. Read the actual worker-owned global
storage where used, rather than an out-of-date shared copy.

Every value object has these required fields:

| Field | Meaning |
| --- | --- |
| `type` | Declared type display name, or the known runtime type |
| `kind` | `integer`, `real`, `boolean`, `string`, `unit`, `record`, `enum`, `array`, `dict`, `option`, `result`, `function`, `task`, `opaque`, or `unknown` |
| `status` | `available`, `uninitialized`, `unavailable`, or `truncated` |
| `scalar` | Scalar payload, or `null` |
| `summary` | Bounded human-readable value/constructor description |
| `children` | Array of `{name, value}` objects, or `[]` |
| `total_children` | Integer count, or `null` when unknown |
| `reason` | Stable omission reason, or `null` |

FPAS integer payloads are canonical decimal **strings**, preserving the complete
signed 64-bit range in ordinary JSON readers. Real payloads are finite JSON
numbers, Booleans are JSON Booleans and strings are JSON strings. Task-handle
payloads use the referenced runtime task id as a JSON integer. Record fields,
enum payloads, array elements and dictionary entries retain declaration/input
order. Dictionary children are consecutive `[0].key`, `[0].value`, `[1].key`,
`[1].value` pairs, with `total_children` equal to twice the entry count. Retain
whole pairs under the child limit. Keys are values, not JSON object property
names. Option/Result/enum summaries name the active
constructor, with children only for its actual payload.

Functions expose their known routine name, receiver and capture values within
the same bounds, without calling them. Hosted resources such as channels and
connections remain typed opaque summaries. Do not drain channels, traverse
native resource internals, evaluate properties, call user code or expose host
addresses. Detect reference cycles and emit a bounded unavailable value with
`reason: "cycle"`. Missing metadata uses `missing_metadata`, not guessed names.
Lock contention uses `busy`, and expired/unavailable storage uses
`storage_unavailable`. Frame absence uses `no_live_frame`; collection expiry
uses `collection_deadline`. Limit reasons match the limits in the next section.

For example, a complete report with one failing task can look like this. The
large integer payload is deliberately a string:

```json
{
  "schema_version": 1,
  "error": {
    "code": "FP5010",
    "message": "panic: invalid total",
    "hint": null,
    "task_id": 0,
    "location": {"source_id": 0, "line": 7, "column": 3}
  },
  "related_errors": [],
  "sources": [{"id": 0, "path": "src/main.fpas"}],
  "capture": {
    "consistency": "cooperative",
    "incomplete": false,
    "reasons": [],
    "total_tasks": 1,
    "omitted_tasks": 0,
    "globals_status": "available",
    "limits": {
      "tasks": 1024,
      "frames_per_task": 256,
      "depth": 16,
      "children_per_value": 256,
      "string_chars": 4096,
      "values": 16384,
      "retained_bytes": 8388608,
      "bytes": 8388608,
      "peer_collection_ms": 500
    }
  },
  "globals": [],
  "tasks": [{
    "id": 0,
    "name": "main",
    "state": "failed",
    "observation": "fault",
    "inspection": "available",
    "unavailable_reason": null,
    "total_frames": 1,
    "frames": [{
      "depth": 0,
      "function": "Example",
      "location": {"source_id": 0, "line": 7, "column": 3},
      "truncated": false,
      "bindings": [{
        "name": "Total",
        "kind": "local",
        "scope_id": 0,
        "value": {
          "type": "integer",
          "kind": "integer",
          "status": "available",
          "scalar": "9223372036854775807",
          "summary": "9223372036854775807",
          "children": [],
          "total_children": 0,
          "reason": null
        }
      }]
    }]
  }]
}
```

## Fixed bounds and completeness

These are feature constants, with no further CLI tuning flags in version 1:

| Bound / omission reason | Limit |
| --- | ---: |
| `task_limit` | 1,024 task records |
| `frame_limit` | 256 frames per task |
| `depth_limit` | Value depth 16; binding root depth is zero, deeper children are omitted |
| `child_limit` | 256 children per value |
| `string_limit` | 4,096 Unicode scalars per captured string |
| `value_limit` | 16,384 value objects across the whole report |
| `retained_byte_limit` | 8 MiB of charged capture data across retained failure candidates per VM |
| `byte_limit` | 8 MiB for the complete UTF-8 document, including escaping and newline |
| `collection_deadline` | 500 ms total for peer coordination, never per task |

`capture` contains `consistency: "cooperative"`, `incomplete` (Boolean),
`reasons` (unique stable omission reasons in lexical order), `total_tasks`,
`omitted_tasks`, and `globals_status` (`available`, `partial`, `unavailable`).
It also contains `limits`, with numeric keys `tasks`, `frames_per_task`,
`depth`, `children_per_value`, `string_chars`, `values`, `retained_bytes`, `bytes` and
`peer_collection_ms`, set to the constants above. Known totals count the whole
available collection, including omitted entries.

Retain the primary diagnostic and task first, then main if distinct, then other
cause tasks, then other tasks in ascending id order. Primary-task frame values
take precedence over peers; globals follow primary frame values. Final task
output uses ascending id order. Retain diagnostic identity/location and a short
message even when user-controlled messages or values exceed the byte budget;
mark clipped messages with `byte_limit` in `capture.reasons`. A selected error's
available-data omissions cannot hide that it was selected.

`incomplete` is true for every omitted/truncated requested datum, including
missing metadata, busy cells, unknown peer frames and unavailable globals.
An opaque value or a completed peer's absent stack is an intentional leaf and
does not by itself make the report incomplete. Cooperative observation alone
does not imply missing data. The deadline bounds peer collection, not existing
program cleanup or arbitrary host filesystem calls.

Apply budgets while reading and constructing the model, before recursive
copies or unbounded allocation. A bounded final document must still be valid
JSON; never cut serialized bytes. A capture failure must preserve the original
diagnostic and, when possible, publish a minimal report marked incomplete.
Candidate captures share the VM-wide retained-byte and value budgets; do not
allocate a full report budget for every unconsumed failed task. Account for
owned model/string/collection storage before allocating it. Preserve compact
causal identities in their existing failure owners when a budget prevents
retaining details, then mark that cause's inspection incomplete if selected.
No instruction history or snapshots on every successful instruction are kept.
No capture state, worker catalog or polling specific to D1 is enabled without
the option. Benchmark successful executions with capture both disabled and
enabled; do not claim a zero-cost implementation before measuring it.

## Acceptance and verification of this step

The contract covers CLI activation, fresh-path publication, exit/diagnostic
behavior, escaping versus handled failures, causal identity, callback stacks,
all relevant tasks, shutdown ordering, typed values, missing data, limits and
metadata-free images. No unresolved product decision blocks step 02.

Checked against the existing [CLI contract](../../pascal/program-structure/cli.md),
[diagnostic codes](../../../crates/fpas-diagnostics/src/codes.rs),
[scope failure selection](../../../crates/fpas-vm/src/vm/tasks/scopes/failures.rs),
[worker execution](../../../crates/fpas-vm/src/vm/worker/run.rs),
[pool execution](../../../crates/fpas-vm/src/vm/tasks/pool.rs) and
[inspection limits](../../../crates/fpas-vm/src/vm/debug/inspection/model.rs).
The existing FP4137/FP4139 diagnostics and runtime exit code 2 cover the new
CLI behavior; no new language rule or diagnostic-number allocation is needed.
Remaining implementation choices must satisfy this contract and be recorded in
their owning step. Completing this document does not claim runtime behavior.

At the step-01 checkpoint, all 102 relative file links in the changed planning files
resolved, the complete JSON example parsed with `Std.Json`, and
`git diff --check` passed. The overview marked only step 01 complete, with five
implementation work packages still open. Planned CLI commands were not executed
as existing functionality; no compiler or runtime source was changed.
