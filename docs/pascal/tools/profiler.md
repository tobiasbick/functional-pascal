# Profile reports

`fpas run --profile <new-file>` runs a program normally and then writes one JSON
report of where its execution time went. It is designed for agents: one command,
one bounded document with stable field names and short findings that name the
dominant cost.

```sh
fpas run app.fpasprj --profile .temp-data/profile.json -- input.txt
```

Program stdout, stderr, diagnostics and exit codes are unchanged. A failing
program still writes its report; its `status` is then `error`. The destination
must be a new file, like [`--failure-dump`](failure-dump.md): an existing path,
`-` or an empty path is rejected before the program starts, and the report is
published atomically. The report never contains program values.

## Report

| Field | Content |
| --- | --- |
| `format`, `version` | `fpas-profile`, `1` |
| `status`, `exit_code` | `ok` or `error`, and the process exit code |
| `wall_ms` | Wall time of the run, excluding compilation |
| `instructions`, `sampled_ms` | Dispatched steps and dispatch time summed over all worker threads |
| `attribution` | How samples are attributed (see below) |
| `routines` | Top 25 routines by self time |
| `lines` | Top 25 source lines by time |
| `counters` | Runtime event counters by name |
| `findings` | Ordered statements about dominant costs, possibly empty |

Rows rank by time because one instruction can allocate, validate a graph or call
native code; instruction counts are deterministic and suit comparisons between
runs. Routine rows have `name` (canonical, case-folded), `source`, `line` (first
statement), `self_instructions`, `self_percent`, `self_ms`,
`inclusive_instructions`, `inclusive_percent` and `inclusive_ms`. Line rows have
`source`, `line`, `routine`, `instructions`, `percent` and `ms`. Sources are
relative to the current directory when possible; `null` when unknown.

## Attribution

The VM dispatches instructions in batches of at most 256. Each batch's steps and
wall time are attributed to the line and routine active when the batch ends
(self) and to every distinct routine on its call stack (inclusive). Rows are
therefore exact in total and statistically accurate per routine; a routine much
shorter than a batch is attributed to its neighbours. Milliseconds include native
intrinsic work called from FPAS code, but not blocked waits, collection or root
publication; those appear in counters.

## Counters

| Counter | Meaning |
| --- | --- |
| `classify_calls`, `classify_visited` | Ownership classification walks and allocations visited |
| `collection_passes`, `collection_nanos`, `collection_visited` | Serviced collection passes, their time and traced values |
| `safe_point_nanos` | Root publication and safe points after batches, including collection |
| `task_spawns`, `task_yields`, `task_parks` | Spawned tasks, time-slice yields and cooperative suspensions |
| `queue_contended`, `worker_idle_nanos` | Contended ready-queue locks and time pool workers waited for work |
| `cancellation_lookups` | Cancellation-token lookups that missed the worker cache |

Counters are process-wide and only count while a profiled run executes.

## Findings

A finding is reported when collection or root publication takes at least 10 % of
execution time, classification visits at least 1,000 allocations per call on
average, yields exceed spawned tasks by more than 100 times, pool workers are idle
for half of their available time while tasks exist, one routine takes at least
half of the sampled time, or one of the five slowest routines spends at least ten
times the average time per instruction while taking at least 10 % of the time.
Each finding states its evidence and a next step.

## Agent workflow

1. Profile the workload once and read `findings`, then the first `routines` rows.
2. If a finding names a runtime mechanism, vary its CLI option (for example
   `--gc-poll-interval`) or the workload size and profile again to confirm it.
3. Optimize the named routine or line, then compare `instructions` and `wall_ms`
   between reports of the same workload on the same machine.
