# UMB-30 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V30-01` | Entry return validates exact result type before mutation | VM positive/negative tests | pass |
| `V30-02` | Child completion publishes one result, wakes eligible waiters, and emits one exit without executing them | VM scheduler/event tests | pass |
| `V30-03` | Root completion terminates root and cancels remaining tasks/timers deterministically | VM task-runtime tests | pass |
| `V30-04` | Stale frame/task, ineligible state, bad value, and unsupported boundary reject atomically | VM snapshot/state comparison tests | pass |
| `V30-05` | Runtime recovery compares the exact unconsumed failure and preserves the original diagnostic | VM failure/recovery tests | pending |
| `V30-06` | Consumed/stale failure and unsupported failure class cannot partially recover | VM negative tests | pending |
| `V30-07` | Retained completed-result replacement is exact and typed, or its missing invariant is recorded | Scheduler/VM tests or decision evidence | pending |
| `V30-08` | Frame restart retains current args and exact captures, clears locals/temporaries, and removes younger frames | VM frame/register/capture tests | pending |
| `V30-09` | Restart executes no instruction; repeated side effects occur only after explicit continue | VM event/side-effect tests | pending |
| `V30-10` | Source-initializer suppression uses portable binding/store identity, never a heuristic | Metadata round-trip and VM tests or decision evidence | pending |
| `V30-11` | Instruction destinations preserve CFG, initialized-register, type, scope, and source invariants | Verifier/dataflow tests or decision evidence | pending |
| `V30-12` | JSONL and DAP expose equivalent accepted results, stable errors, and events | Paired adapter tests | pass |
| `V30-13` | VS Code exposes only supported lifecycle operations and refreshes stopped state correctly | Extension-host tests | pass |
| `V30-14` | Current docs describe implemented behavior and limitations only | Documentation review | pass for U30-10/11 |
| `V30-15` | No FPAS syntax, semantic, or language-spec change | Diff review | pass for U30-10/11 |
| `V30-16` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pass for U30-10/11 |

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
