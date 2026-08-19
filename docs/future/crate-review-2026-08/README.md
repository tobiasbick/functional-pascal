# Crate review decisions (2026-08)

The implementation-ready and coverage findings from the workspace crate review are complete.
Their current behavior, tests, and user-facing documentation live with the owning crates and under
`docs/pascal/`.

This directory retains only findings whose intended behavior still needs an explicit user decision.
Read [`how-to-implement.md`](how-to-implement.md) before starting one. Record the selected option in
the task before changing code or current user-facing documentation.

## Decision queue

| Task | Decision |
|---|---|
| [14 named record identity](tasks/14-sema-named-records.md) | Nominal or structural compatibility for distinct named records |
| [18 public API using a private type](tasks/18-sema-export-private-type.md) | Reject the declaration or support opaque public use |
| [21 Sleep/Yield in synchronous callbacks](tasks/21-vm-callback-sleep.md) | Cooperatively suspend the owner or block the current worker |
| [23 overlapping consumer/library source](tasks/23-project-origin.md) | Reject overlap or define which project owns the source |
| [26 test timeout policy](tasks/26-cli-test-timeout.md) | Timeout scope and default duration |

No task in this queue is implementation-ready until its decision record names the selected option
and the user's approval.
