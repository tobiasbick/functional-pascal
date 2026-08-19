# Crate review decisions (2026-08)

The defect and coverage findings from this review have been implemented and removed from future
planning. The remaining findings are semantic or policy questions that cannot be resolved from the
current specification.

| Area | Question | Task |
|---|---|---|
| Sema | Are distinct named records nominally incompatible or structurally compatible? | [14](tasks/14-sema-named-records.md) |
| Sema | Must a public declaration that mentions a private type be rejected, or is opaque public use supported? | [18](tasks/18-sema-export-private-type.md) |
| VM | Does `Sleep` inside a synchronous callback suspend the owner cooperatively or block its worker? | [21](tasks/21-vm-callback-sleep.md) |
| Project | Is a physical source shared by a consumer and dependency rejected, consumer-owned, or library-owned? | [23](tasks/23-project-origin.md) |
| CLI | Does the test timeout include worker startup, and is there a default timeout? | [26](tasks/26-cli-test-timeout.md) |
