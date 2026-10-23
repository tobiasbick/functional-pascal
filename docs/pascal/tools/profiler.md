# Profile reports

`fpas run --profile <new-file>` runs a program normally and then writes one JSON
report of where its execution time went. It is designed for agents: one command,
one bounded document with stable field names, explicit units and findings that
carry their measurements.

```sh
fpas run app.fpasprj --profile .temp-data/profile.json -- input.txt
```

Program stdout, stderr, diagnostics and exit codes are unchanged. A failing
program still writes its report; its `status` is then `error`. The destination
must be a new file, like [`--failure-dump`](failure-dump.md): an existing path,
`-` or an empty path is rejected before the program starts, and the report is
published atomically. The report never contains program values, program
arguments, environment variables or absolute paths.

## Report

| Field | Content |
| --- | --- |
| `format`, `version` | `fpas-profile`, `2` |
| `status`, `exit_code` | `ok` or `error`, and the process exit code |
| `wall_ms` | Wall time of the run, excluding compilation |
| `sampled_ms` | Dispatch time summed over all threads executing FPAS code |
| `parallelism` | `sampled_ms / wall_ms`: average number of threads executing FPAS code |
| `instructions` | Dispatched scheduling steps, mostly instructions |
| `run` | Tool, target, runtime and sampling metadata (below) |
| `routines`, `lines` | `{ "total": N, "rows": [...] }`: eligible row count and the top rows by time |
| `counters` | Runtime event counters by name |
| `findings` | Structured findings, possibly empty |

Routine rows have `name` (canonical, case-folded), `source`, `line` (first
statement), `self_instructions`, `inclusive_instructions`, `self_ms`,
`inclusive_ms`, `self_time_percent` and `inclusive_time_percent`. Line rows have
`source`, `line`, `routine`, `instructions`, `ms` and `time_percent`. Time shares
divide by `sampled_ms`; instruction shares follow from the counts and
`instructions`. Inclusive rows overlap and do not sum to 100 %. Rows rank by
self time because one instruction can allocate, validate a graph or call native
code; instruction counts are deterministic and suit comparisons between runs.
Sources are relative to the current directory when possible; `null` when unknown.
`total` above the row limit means rows were omitted.

### Run metadata

| Field | Content |
| --- | --- |
| `run.tool` | `version` and the compiler `build` identity |
| `run.target` | `kind` (`source`, `project`, `workspace`, `compiled`), `name` relative to the current directory or the file name, and `digest`: `blake3:` over the executed bytecode including its debug metadata |
| `run.runtime` | Effective pool `workers` and collection settings (`cell_threshold`, `mutation_threshold`, `poll_interval`), defaults included |
| `run.sampling` | `method`, `max_batch_instructions`, `row_limit` and `counters_exclusive` |

Equal digests and settings are necessary for a controlled comparison, but do not
prove equal arguments, input files, network responses, machine load or power
settings; control those separately. Compare profiled runs only with other
profiled runs: their wall times do not measure the overhead of unprofiled runs.

## Attribution

The VM dispatches instructions in batches of at most `max_batch_instructions`.
Each batch's steps and wall time are attributed to the line and routine active
when the batch ends (self) and to every distinct routine on its call stack
(inclusive). Rows are therefore exact in total and statistically accurate per
routine; a routine much shorter than a batch is attributed to its neighbours.
Milliseconds include native intrinsic work called from FPAS code, but not blocked
waits, collection or root publication; those appear in counters.

## Counters

| Counter | Meaning |
| --- | --- |
| `classify_calls`, `classify_visited` | Ownership classification walks and shared values visited |
| `collection_passes`, `collection_nanos`, `collection_visited` | Serviced collection passes, their time and traced values |
| `safe_point_nanos` | Root publication and safe points after batches, including collection |
| `task_spawns`, `task_yields`, `task_parks` | Spawned tasks, time-slice yields and cooperative suspensions |
| `queue_contended`, `worker_idle_nanos` | Contended ready-queue locks and time pool workers waited for work |
| `cancellation_lookups` | Cancellation-token lookups that missed the worker cache |

Counters are process-wide. `fpas run` profiles one VM; an embedding host that
profiles several VMs at once shares the counters between them, which the report
shows as `counters_exclusive: false`.

## Findings

Each finding is an object with a stable `code`, a short `message`, `evidence`
(measurements with `name`, `value`, `unit` and, for shares and ratios,
`denominator`), an optional `location` (`routine`, `source`, `line`) and
`next_steps` (suggested checks). Messages state observations; next steps suggest
how to confirm a cause. Findings appear in this order, at most eight:

| Code | Triggered when |
| --- | --- |
| `collection_time` | Collection takes at least 10 % of `busy_ms` (`sampled_ms` plus safe points) |
| `safe_point_time` | Root publication outside collection takes at least 10 % of `busy_ms` |
| `classification_walks` | Classification visits at least 1,000 values per call on average |
| `task_yields` | Time-slice yields exceed spawned tasks by more than 100 times |
| `worker_idle` | Pool workers are idle for at least half of `workers * wall_ms` while tasks exist |
| `dominant_routine` | The slowest routine other than the program entry takes at least half of `sampled_ms` |
| `costly_instructions` | One of the five slowest routines spends at least ten times the average time per instruction and at least 10 % of `sampled_ms` |

## Agent workflow

1. Profile the workload and read `findings`, then the first `routines` and `lines`
   rows. Low `parallelism` with tasks points at serial phases or unequal tasks.
2. If a finding names a runtime mechanism, vary its CLI option (for example
   `--gc-poll-interval`) or the workload size and profile again to confirm it.
3. Before comparing two reports, check that `run.target.digest`, `run.runtime`
   and `run.sampling` match and `counters_exclusive` is true. Then compare
   `instructions`, `sampled_ms` and `wall_ms` of the same workload on the same machine.
