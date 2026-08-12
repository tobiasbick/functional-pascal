# Implementation plan

Stable work-package IDs remain valid across resumed sessions.

| ID | Work package | State | Exit gate |
|---|---|---|---|
| UBA-01 | Add explicit per-register initialization state and central register store/take operations | implemented | Unit, call, callback, intrinsic, frame-reuse, and task-state regressions |
| UBA-02 | Capture empty mutable locals/globals as root mutation targets without exposing descendants | implemented | Session tests distinguish empty storage from initialized `unit` |
| UBA-03 | Validate and atomically commit complete root assignments | implemented | Type, cancellation, limit, immutability, stale-handle, and descendant failures preserve storage |
| UBA-04 | Map behavior through JSONL and DAP | implemented | Protocol transcripts cover handle and textual entry points plus invalidation |
| UBA-05 | Exercise the installed VS Code adapter path | implemented | Extension Host test observes empty local, forwards both standard requests, and continues correctly |
| UBA-06 | Reconcile current docs, deferred scope, and verification evidence | verified | Documentation links resolve and all commands in `progress.md` pass |

## Resume procedure

1. Confirm the current branch and preserve unrelated working-tree changes.
2. Read `git diff --stat` and this table.
3. Run the narrow tests named in `verification-matrix.md` before broad gates.
4. Fix only failing matrix rows; do not enlarge the accepted scope.
5. Update `progress.md` with the exact command and result after each gate.
6. Mark UBA-06 complete only after formatting, build, workspace tests, FPAS
   formatting, and the relevant VS Code test command succeed.
