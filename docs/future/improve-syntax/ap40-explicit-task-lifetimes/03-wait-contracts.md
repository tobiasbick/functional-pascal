# AP40.3: Wait contracts

Package: [AP40](README.md). Status: blocked by AP40.1's wait decision.

## Scope and prerequisites

Implement the selected ready/timeout/cancellation policy and consistent
completion/result terminology. Depend on AP40.2 only if the selected API
actually needs its new ownership behavior; record that before starting.
Do not turn completion observation into result consumption accidentally.

## Affected areas, migration, and documentation

Inspect hosted task/group/selection implementations under
`crates/fpas-vm/src/`, task builtins under `crates/fpas-std/src/`, and typed
call validation. Update [Std.Tasks](../../../pascal/std/concurrency/task.md)
and concurrency examples. Migrate callers affected by changed boundary
priorities and teach storing a consumed result exactly once. Avoid a new
linear type system solely to rename or clarify Wait.

## Verification

Use controlled readiness and cancellation signals to test first/subsequent
observations, simultaneous events, elapsed deadlines, Select case order,
multiple handle aliases, repeated consuming waits, and non-consuming waits
followed by retrieval. No long sleeps or timing-only assertions as proof of
priority. Run relevant runtime/FPAS and repository checks.
