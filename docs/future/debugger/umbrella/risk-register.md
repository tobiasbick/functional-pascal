# Umbrella risk register

| ID | Risk | Affected packages | Mitigation | Stop trigger | State |
|---|---|---|---|---|---|
| `UMB-R01` | Display names or rendered values are mistaken for runtime identity | `10`, `20`, `60`, `70`, `80`, `90` | Require numeric/structural metadata identity and collision tests | Any successful operation relies on display parsing | open |
| `UMB-R02` | Task control observes or mutates a non-quiescent shared state | `30`, `40`, `50`, `70`, `80` | Complete `UMB-40A` before dependent mutation or history | A supposedly stopped peer can still mutate observed state | open |
| `UMB-R03` | Debuggee terminal/TUI bytes corrupt protocol transport | `50`, `60` | Separate authenticated debuggee channel from JSONL/DAP stdio | Protocol and debuggee share an unframed stream | open |
| `UMB-R04` | A failed control or mutation request partially changes execution | `10`, `30`, `40`, `70`, `90` | Prepare/validate/commit transaction and rollback tests | Failure changes frames, tasks, values, handles, or generation | open |
| `UMB-R05` | JSONL, DAP, and VS Code acquire divergent policy | all | Implement VM first, then JSONL, DAP, and editor mapping | Adapter accepts a request rejected by shared policy | open |
| `UMB-R06` | Required identity is absent from portable debug metadata | `10`, `20`, `60`, `70`, `90` | Prove compiler-to-program round trip before runtime use | Runtime would need name guessing or source re-parsing | open |
| `UMB-R07` | History, recordings, snapshots, or output grow without bound | `40`, `50`, `60`, `80`, `90` | Explicit count/byte/time retention limits and cleanup | Any default path retains unbounded state | open |
| `UMB-R08` | Replay claims determinism while host or scheduler effects are missing | `80` | Inventory effects and reject unsupported recordings up front | Same recording can produce different visible state | open |
| `UMB-R09` | Hot reload corrupts active layouts, closures, frames, or tasks | `90` | Versioned compatibility proof and recoverable old image | An incompatibility is detected only after commit | open |
| `UMB-R10` | Attach or remote debug exposes paths, sources, or control without authorization | `60`, `80`, `90` | Authentication, source mapping, redaction, explicit ownership | Unauthenticated discovery/control or raw host metadata exposure | open |
| `UMB-R11` | Unrelated workspace failures hide debugger regressions | `00`, `99` | Record exact baseline targets and run focused gates first | A new failure cannot be separated from the baseline | open |
| `UMB-R12` | Umbrella scope becomes one unreviewable patch | all | One active package, coherent checkpoint commits, fixed exit gates | More than one dependent primary package is implemented without a gate | open |

## Risk handling

- `open`: mitigation is required by the owning package.
- `controlled`: current tests prove the mitigation for the active package.
- `triggered`: stop implementation and update the contract or dependency map.
- `closed`: the final acceptance matrix proves no remaining package exposure.

A triggered risk is not automatically a permanent deferral. Investigate it,
record evidence in `progress.md`, and then either revise the active package or
return a genuinely independent unresolved capability to `../deferred.md`.

