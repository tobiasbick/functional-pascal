# Runtime failure reports

`fpas run --failure-dump PATH` writes one bounded JSON document when a runtime
diagnostic makes the run fail. It works with source files, projects, workspaces,
direct `.fpascp` images and project/workspace discovery.

```text
fpas run main.fpas --failure-dump .temp-data/failure.json
fpas run --diagnostics json --failure-dump .temp-data/failure.json app.fpascp -- input.txt
```

Choose a **new file path** for each run. Relative paths use the invocation
directory. Empty paths, `-`, a repeated option, a missing value and an existing
destination (including a directory or symlink) fail before execution with FP4137.
Only `run` accepts the option; arguments after `--` remain program arguments.
There is no default filename or manifest setting.

Successful execution, handled task failures, ordinary `Error`/`None` values and
compilation/load/argument failures create no report, staging file or report
parent directories. A qualifying runtime failure creates missing parents, writes
and syncs a sibling staging file, then atomically publishes a hard link to the
new destination. A racing destination is preserved. The filesystem must support
hard-link publication; failure is reported instead of using a non-atomic fallback.
Owned staging files are removed on success and attempted cleanup follows errors.
Process kills, out-of-memory termination, host aborts and native crashes end the
process without a report.

Program stdout and the selected text/JSON stderr diagnostic format are preserved.
Successful publication prints no extra record. Runtime failure returns exit code
**2**, including publication failure; the latter adds FP4139 after the original
runtime diagnostic. Success returns **0** and pre-execution failures return **1**.

## Observation and ownership

The cause's frames and values are retained at the original fault, before its
worker unwinds. The task catalog is a later cooperative cut during failure
handling. It contains main (id `0`), known causes, live tasks and retained terminal
outcomes. Released task history is not reconstructed. A task can be listed once
even when several errors refer to it. The primary cause's stack takes precedence.

Running peers answer at existing dispatch boundaries; saved state is inspected
under its owner's synchronization. Collection never executes queued code, consumes
results/channels, calls a property or user function, or switches to the debugger
scheduler. One 500 ms deadline covers all peers. A blocked native peer retains its
known identity/state with unavailable frames. Answering peers continue immediately.
Synchronous callback observations disclose partial outer storage where necessary.

An independent writer receives immutable data after capture coordination has been
released. Publication can finish before remaining VM shutdown joins. Semantic
ownership barriers still run before the escaping error is selected; an unfinished
owned native operation can delay that selection. Reports do not bypass cleanup or
detect deadlocks. Report errors do not change normal program termination guarantees.

## JSON version 1

The file contains one UTF-8 object and a trailing newline. Check `schema_version`
before reading it. Object member order is insignificant. Required keys remain
present; unavailable optional data is `null`.

| Top-level key | Meaning |
| --- | --- |
| `schema_version` | Integer `1` |
| `error` | Selected diagnostic object |
| `related_errors` | Other attached diagnostics, flattened in attachment order |
| `sources` | Source identities used by delivered error/frame positions |
| `capture` | Completeness, known task counts and fixed bounds |
| `globals` | Shared global bindings, captured once |
| `tasks` | Selected task entries in ascending runtime ID order |

An **error** has `code` (`FPnnnn`), `message`, `hint` (string or `null`),
`task_id` (integer or `null`), and `location`. A **location** has `source_id`,
`line` and `column`, with one-based coordinates. Unknown positions are `null`.
Source entries have `id` and portable `path` (or `null`). Paths use `/`; absolute
compiler paths retain only their basename. Source IDs distinguish equal basenames.
Path metadata has a separate 1 MiB copy allowance and a 4096-character limit per
path. If a name is omitted, known source IDs and coordinates remain intact with
`path: null`; the capture reports the corresponding omission reason. No source
file is read to manufacture missing debug information.

A **task** has `id`, `name` (string or `null`), `state`, `observation`,
`inspection`, `unavailable_reason` (string or `null`), `total_frames` (integer or
`null`) and `frames`. States are `runnable`, `running`, `waiting`, `sleeping`,
`completed`, `failed` and `cancelled`; running includes native work. Observations
are `fault` or the later `failure_handling`. Inspection is `available`, `partial`,
`unavailable` or `not_applicable`. Completed/cancelled peers have no fabricated
stack. Retained failed peers can expose their original fault frames.

A **frame** has `depth` (active frame zero, callers outwards), `function`,
`location`, `bindings` and `truncated`. Only actual FPAS frames are represented;
tail-call optimization can remove a caller. Each **binding** has `name`, `kind`
(`parameter`, `local`, `capture` or `global`), `scope_id` (integer or `null` for
globals) and `value`. Arrays preserve duplicate names; lexical scope IDs keep
shadowed bindings distinct. Compiler temporaries and out-of-scope bindings are
excluded. Uninitialized bindings remain explicitly uninitialized; `var` parameters
show the current referent. Globals use the actual main worker's local/shared slots.
Images without debug bindings retain stored diagnostic positions and identify
missing bindings with `missing_metadata`; locals are not inferred from registers.

A known task entry name survives `Wait` and result release with its original
failure, including when a tail call has removed that entry's frame. It is not
inferred from the remaining stack.

Every **value** has these keys:

| Key | Meaning |
| --- | --- |
| `type` | Declared display type, or known runtime type |
| `kind` | `integer`, `real`, `boolean`, `string`, `unit`, `record`, `enum`, `array`, `dict`, `option`, `result`, `function`, `task`, `opaque` or `unknown` |
| `status` | `available`, `uninitialized`, `unavailable` or `truncated` |
| `scalar` | Scalar payload, or `null` |
| `summary` | Bounded description without evaluating user code |
| `children` | Ordered array of `{name, value}` objects |
| `total_children` | Known immediate count, or `null` |
| `reason` | Stable omission reason, or `null` |

Integer payloads are canonical decimal **strings**, preserving the complete signed
64-bit range. Finite reals are JSON numbers, Booleans JSON Booleans, strings JSON
strings and task handles JSON integer IDs. Record/enum payloads retain declaration
order. Dictionary children are consecutive `[0].key`, `[0].value`, `[1].key`,
`[1].value` pairs, never JSON object members; `total_children` is twice the entry
count. Bounds retain complete pairs. Option/Result/enum summaries identify their
active constructor and expose only its payload. Functions expose known routine,
receiver and capture data within the same bounds. Native resources remain typed
opaque leaves; their internals are not traversed.

## Bounds and completeness

`capture` contains `consistency: "cooperative"`, `incomplete`, lexically sorted
unique `reasons`, `total_tasks`, `omitted_tasks`, `globals_status` and `limits`.
Task counts include known omitted identities; counts can be `null` in a minimal
fallback whose catalog could not be captured. Globals status is `available`,
`partial` or `unavailable`.

| `limits` key | Value | Omission reason |
| --- | ---: | --- |
| `tasks` | 1024 | `task_limit` |
| `frames_per_task` | 256 | `frame_limit` |
| `depth` | 16 child edges, binding root depth zero | `depth_limit` |
| `children_per_value` | 256 | `child_limit` |
| `string_chars` | 4096 Unicode scalars | `string_limit` |
| `values` | 16384 across VM captures | `value_limit` |
| `retained_bytes` | 8388608 charged bytes across retained VM captures | `retained_byte_limit` |
| `bytes` | 8388608 complete UTF-8 document bytes, including escaping/newline | `byte_limit` |
| `peer_collection_ms` | 500 total | `collection_deadline` |

Other reasons are `missing_metadata`, `busy`, `storage_unavailable`, `capture_failed`,
`no_live_frame` and `cycle`. `incomplete` is true for missing or truncated requested
data. An opaque leaf, a completed peer's absent stack or cooperative timing alone does not imply
missing data. Original capture reasons remain in the report even when document
size requires additional clipping.

Planning retains primary/main identities, other cause identities and attached
diagnostics before optional values. Primary values precede globals and peer
values; main and other cause frames precede remaining peers in ascending ID order.
Strings/child arrays can be shortened at JSON boundaries, with `truncated` values,
original known child counts and `byte_limit` markers. Frame/task/global omissions
are disclosed too. The file is always complete JSON; serialized bytes are never
cut. Capture/serialization failure attempts a minimal incomplete report preserving
the selected diagnostic. No instruction history or released failure history is kept.
Capture remains disabled unless the option is supplied.

## Example

Save this as `main.fpas`, then run the first command above with a fresh report path:

```pascal id=failure-example run error=FP5010
program FailureExample;

begin
  const Total := 42;
  panic('invalid total');
end.
```

Its report contains:

```json
{
  "schema_version": 1,
  "error": {
    "code": "FP5010",
    "message": "panic: invalid total",
    "hint": "Remove the panic or guard the failing condition before calling panic.",
    "task_id": 0,
    "location": {"source_id": 0, "line": 5, "column": 3}
  },
  "related_errors": [],
  "sources": [{"id": 0, "path": "main.fpas"}],
  "capture": {
    "consistency": "cooperative",
    "incomplete": false,
    "reasons": [],
    "total_tasks": 1,
    "omitted_tasks": 0,
    "globals_status": "available",
    "limits": {
      "tasks": 1024, "frames_per_task": 256, "depth": 16,
      "children_per_value": 256, "string_chars": 4096, "values": 16384,
      "retained_bytes": 8388608, "bytes": 8388608, "peer_collection_ms": 500
    }
  },
  "globals": [],
  "tasks": [{
    "id": 0, "name": "main", "state": "failed", "observation": "fault",
    "inspection": "available", "unavailable_reason": null, "total_frames": 1,
    "frames": [{
      "depth": 0, "function": "failureexample",
      "location": {"source_id": 0, "line": 5, "column": 3},
      "bindings": [{
        "name": "Total", "kind": "local", "scope_id": 0,
        "value": {
          "type": "integer", "kind": "integer", "status": "available",
          "scalar": "42", "summary": "42", "children": [],
          "total_children": 0, "reason": null
        }
      }],
      "truncated": false
    }]
  }]
}
```

See [CLI](../program-structure/cli.md), [shared diagnostics](diagnostics.md) and
[debugger](debugger.md) for the surrounding tools and Rust embedding API.
The [local benchmark guide](../../bench/failure-dump.md) describes reproducible
disabled/enabled comparisons and their measurement boundaries.
