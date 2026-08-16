# UMB-40 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V40-01` | While stopped or failed, no task executes `dispatch_one` | VM stop-state tests | pass |
| `V40-02` | Every stop freezes live peers at instruction boundaries; completed/cancelled tasks have no fabricated stack | VM catalog/stack tests | pass |
| `V40-03` | Shared cells and globals observed at a stop are unchanged by supposedly stopped peers | VM shared-state tests | pass |
| `V40-04` | Scheduler waiters and timers do not dispatch bytecode during a stop | VM/scheduler tests | pass |
| `V40-05` | Pause requested during an in-progress host intrinsic is observed after it returns | Existing cooperative-pause test plus documented bound | pass |
| `V40-06` | Breakpoint, failure, pause, or limit from any progressed task wins over the step target | Existing dependency-priority tests plus gaps closed | pass |
| `V40-07` | Per-task pause/resume rejects unknown, stale, and terminal IDs without mutation | VM negative tests or child-not-started evidence | pass |
| `V40-08` | A paused task does not execute as a hidden wait-dependency | VM peer-execution tests | pass |
| `V40-09` | Create/cancel/restart publish exact result/waiter effects without command-time execution | VM scheduler/event tests or decision evidence | pass |
| `V40-10` | Non-stop, shortcuts, and retained history are proven or rejected with bounds | Feasibility evidence | rejected |
| `V40-11` | JSONL and DAP expose equivalent task identity, stop owner, and all-stop flags | Paired adapter tests | pass |
| `V40-12` | VS Code exposes only supported task operations and all-stop thread state | Extension-host tests | pass |
| `V40-13` | Current docs describe implemented behavior and limitations only | Documentation review | pass |
| `V40-14` | No FPAS syntax, semantic, or language-spec change | Diff review | pending |
| `V40-15` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pending |

## Standard final commands

```powershell
cargo fmt --all -- --check
cargo build --workspace --locked
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings
cargo test --workspace --locked --no-fail-fast
npm test --prefix editors/vscode
git diff --check
```

Add targeted commands to [progress.md](progress.md) as ownership becomes
concrete. If `.fpas` fixtures change, also run their formatter check and the
applicable FPAS test bundle.
