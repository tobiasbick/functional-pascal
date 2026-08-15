# Verification matrix

Status values are `PLANNED`, `PASS`, or `BLOCKED`. A row becomes `PASS` only
after its evidence command and assertion have run against the current checkout.

| ID | Acceptance case | Planned evidence | Status |
|---|---|---|---|
| CCRA-T01 | A direct `Cell` capture is materialized from the exact selected owner binding | Compiler metadata plus VM construction test | PLANNED |
| CCRA-T02 | An `EnclosingCell` capture reuses the same transitive cell identity | Compiler/VM pointer-identity test | PLANNED |
| CCRA-T03 | Mixed immutable and cell captures retain declared ABI order and portable types | Compiler and VM ordered-capture assertions | PLANNED |
| CCRA-T04 | Later writes through the owner are observed by the assigned routine | Source fixture continuation test | PLANNED |
| CCRA-T05 | Writes through the assigned routine are observed by every pre-existing closure sharing the cell | VM and protocol continuation tests | PLANNED |
| CCRA-T06 | The exact recursive owner activation is used; older and similarly named frames are not searched | VM recursive-frame positive/negative cases | PLANNED |
| CCRA-T07 | Uninitialized, hidden, out-of-scope, wrong-kind, wrong-type, missing, duplicate, and malformed capture metadata are rejected | IR/bytecode/artifact validation plus VM negatives | PLANNED |
| CCRA-T08 | Construction clones cell handles without locking or reading their payloads | Busy/poisoned/cyclic cell unit cases | PLANNED |
| CCRA-T09 | Constructed mutable-capture functions are task-bound and carry the selected runtime task owner | Value and VM ownership assertions | PLANNED |
| CCRA-T10 | Existing VM-created mutable closures carry the same task-owner invariant | VM closure creation regression | PLANNED |
| CCRA-T11 | Owner-task invocation succeeds while foreign-task invocation fails before callee entry | VM two-task runtime cases | PLANNED |
| CCRA-T12 | `go` continues to reject every task-bound function | Existing and new task-spawn regressions | PLANNED |
| CCRA-T13 | Compatible mutable local and parameter frame-register roots succeed | VM session cases | PLANNED |
| CCRA-T14 | A compatible uninitialized mutable local frame register accepts one complete value only if existing initialization rules allow it | VM positive/negative cases | PLANNED |
| CCRA-T15 | Global, cell-backed capture, aggregate descendant, inactive payload, Dynamic, evaluation-only, and immutable targets remain rejected | Table-driven VM policy cases | PLANNED |
| CCRA-T16 | Request frame, lexical owner, target frame, stop generation, and task must all match | Stale/foreign/peer frame cases | PLANNED |
| CCRA-T17 | Immutable value captures still obey depth/value limits and reject nested cells, tasks, opaque handles, and task-bound functions | Capture-graph unit cases | PLANNED |
| CCRA-T18 | Signature mismatch and ambiguous/unknown routine names retain actionable stable errors | VM and protocol negatives | PLANNED |
| CCRA-T19 | Failure preserves destination value, cell identities, stopped state, and old frame/variable references | VM atomicity plus transcript assertions | PLANNED |
| CCRA-T20 | Success commits once, refreshes all stopped task snapshots, and expires old references once | VM and protocol generation assertions | PLANNED |
| CCRA-T21 | JSONL `variable.set` and `expression.set` have equivalent success/failure behavior | JSONL integration test | PLANNED |
| CCRA-T22 | DAP standard requests map values, errors, hints, and negotiated invalidation without a custom request | DAP integration test | PLANNED |
| CCRA-T23 | VS Code Variables and Watch assignment continue through shared mutable state | Extension Host test | PLANNED |
| CCRA-T24 | Existing immutable named-routine assignment and non-task-bound function-copy suites do not regress | Focused existing suites | PLANNED |
| CCRA-T25 | `.fpascu`, linker, and `.fpascp` retain capture kinds and owner binding identities without serializing runtime task owners | Unit/linker/program round-trip tests | PLANNED |
| CCRA-T26 | FPAS fixture formatting is stable | `fpas fmt --check` | PLANNED |
| CCRA-T27 | Rust format, Clippy, build, workspace tests, VS Code tests, links, privacy, and diff checks pass | Commands recorded in `progress.md` | PLANNED |

## Required assertions

Success evidence must include canonical function identity, selected task/frame,
exact `Arc` sharing, task owner, continuation result, old-reference expiry, and
one negotiated DAP invalidation. Failure evidence must include stable code and
hint, unchanged destination and cell graph, preserved handles and stop, no
callee entry, and no invalidation.
