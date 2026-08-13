# Verification matrix

Status values are `PLANNED`, `PASS`, or `BLOCKED`. Planning does not qualify a
row as `PASS`.

| ID | Acceptance case | Planned evidence | Status |
|---|---|---|---|
| FRT-T01 | A stopped scalar function returns a supplied compatible value immediately | VM session and source fixture | PASS |
| FRT-T02 | The caller receives the value at its saved destination and subsequent continuation observes it | VM and protocol continuation assertions | PASS |
| FRT-T03 | A stopped procedure returns without an expression and restores its caller | VM and source fixture | PASS |
| FRT-T04 | A function requires an expression; a procedure rejects one | VM, JSONL, and DAP negative cases | PASS |
| FRT-T05 | Detached read-only expressions and allowed controlled calls evaluate exactly once before commit | call counter plus evaluation transcript | PASS |
| FRT-T06 | Scalar and supported immutable aggregate result types validate structurally | type-focused VM cases | PASS |
| FRT-T07 | Type mismatch, Dynamic, function, task, opaque, and unsupported runtime values fail actionably | VM and protocol negative cases | PASS |
| FRT-T08 | Missing or malformed result metadata is rejected without display-name inference | bytecode/object verifier and session cases | PASS |
| FRT-T09 | Result metadata survives compiler output, relocatable object conversion, `.fpascu`, linker translation, and `.fpascp` | compiler/unit/linker/program round trips | PASS |
| FRT-T10 | A stale, foreign, non-depth-zero, or peer-task frame is rejected | VM and JSONL selected-task cases | PASS |
| FRT-T11 | Program entry, task entry, waiting/suspended task, and runtime-error stops remain unsupported | VM lifecycle cases | PASS |
| FRT-T12 | Failure preserves frames, registers, instruction count, stop reason, values, and every inspection handle | before/after identity assertions | PASS |
| FRT-T13 | Success removes exactly one frame and releases only its register window | Worker and session boundary tests | PASS |
| FRT-T14 | Success remains stopped at the caller without dispatching an instruction or scheduler turn | stop/location/count assertions | PASS |
| FRT-T15 | Success refreshes all stopped snapshots and expires old stack/variable handles exactly once | multi-task session and protocol cases | PASS |
| FRT-T16 | JSONL advertises and implements `frame.return` with stable result fields, codes, and hints | JSONL transcript | PASS |
| FRT-T17 | DAP returns equivalent result/type/error metadata and emits one negotiated stacks/variables invalidation only on success | paired JSONL/DAP transcripts | PASS |
| FRT-T18 | VS Code invokes the custom request for the active frame, prompts only when required, and refreshes the caller | Extension Host test | PASS |
| FRT-T19 | Existing return execution, stepping, mutation, task scheduling, callbacks, and artifact loading do not regress | focused existing suites and workspace gate | PASS |
| FRT-T20 | The FPAS fixture is canonical | `fpas fmt --check` | PASS |
| FRT-T21 | Rust formatting, lint, build, workspace tests, editor tests, and diff checks pass | commands in `progress.md` | BLOCKED |

## Traceability rule

Each `PASS` entry must name its concrete test or recorded command in
`progress.md`. If the accepted boundary changes, update the matching FRT-S,
FRT-D, work-package, and test IDs together before implementation continues.
