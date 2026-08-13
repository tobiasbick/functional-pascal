# Progress

Last updated: 2026-08-13

## Current checkpoint

Implemented and re-audited on 2026-08-13. The bounded active-callee
forced-return slice is live on the shared VM debugger and the JSONL, DAP, and
VS Code surfaces. Remaining exclusions stay in `consciously-deferred.md`.

Portable `FunctionDebugInfo.result_type` is retained through compiler, object,
linker, and `.fpascp` encoding. The session evaluates the return expression
only after eligibility, convention, and declared-category checks, then commits
one preflighted frame pop without dispatching a `Return` opcode.

The re-audit moved bounded result rendering and result-handle admission before
the live frame commit, reserves bounded handle capacity in the fresh caller
snapshot, tightens caller instruction-pointer preflight to the half-open code
range, and advances the `.fpascu` envelope version for the changed object
payload. The forced-return tests and all affected crates pass. FRT-08 remains
blocked only because the full workspace gate still contains the unrelated
Notes reference-count failure described below.

## Work-package status

| ID | State | Evidence or next action |
|---|---|---|
| FRT-01 | complete | Fixture `tests/debugger/fixtures/forced_return.fpas` plus VM/JSONL/DAP contract cases |
| FRT-02 | complete | Optional result type through compiler, object validation, linker retention, and program digest |
| FRT-03 | complete | `forced_return/validate.rs` and `commit.rs`; type, rendering, and handle-limit failures preserve registers, frames, counts, and handles |
| FRT-04 | complete | `session/forced_return.rs` refresh, Pause stop, and continuation cases |
| FRT-05 | complete | JSONL `frame.return`, `frame_return` capability, codes, and hints |
| FRT-06 | complete | DAP `fpas/forceReturn`, stacks/variables invalidation, VS Code command |
| FRT-07 | complete | Current `docs/pascal/tools/` pages and editor README describe the implemented command |
| FRT-08 | blocked | Forced-return and affected-crate gates pass; the full workspace command has one unrelated Notes reference-count failure |

## Evidence log

Recorded 2026-08-13:

```text
cargo fmt --all -- --check
# exit 0

cargo clippy -p fpas-bytecode -p fpas-compiler -p fpas-unit -p fpas-linker -p fpas-vm -p fpas-debug --all-features --locked -- -D warnings
# exit 0

cargo build --locked
# exit 0

cargo test -p fpas-bytecode --locked
# exit 0 (25 lib + 27 bytecode integration)

cargo test -p fpas-unit --locked
# exit 0 (object result-type round trip and invalid table reference)

cargo test -p fpas-linker --locked
# exit 0 including linker_retains_result_only_debug_types

cargo test -p fpas-vm --lib forced_return --locked
# 16 passed

cargo test -p fpas-debug --test forced_return --locked
# 3 passed

cargo test -p fpas-debug --test dap_forced_return --locked
# 2 passed

cargo test --workspace --no-fail-fast --locked
# exit 101; sole failure is repository_references_find_notes_update_in_the_consuming_program
# (23 actual vs 22 expected NotesUpdate references). Unrelated to this package.

cargo run -p fpas-cli --locked -- fmt --check tests/debugger/fixtures/forced_return.fpas
# exit 0

cd editors/vscode && npm test
# exit 0

git diff --check
# exit 0 (CRLF conversion warnings only)
```

Focused mapping:

- FRT-T01, T02, T03, T04, T12, T13, T14: `crates/fpas-vm/src/vm/debug/tests/forced_return/`
- FRT-T12: additionally covers pre-commit output and result-handle limit failures
- FRT-T05: VM callee-local `Offset + 40` plus JSONL `PlusOne(41)` before commit
- FRT-T06: VM array structural return
- FRT-T07: VM type/category cases plus JSONL `frame_return_type`
- FRT-T08: bytecode verifier, object validation, metadata-less session case
- FRT-T09: compiler debug metadata, unit object conversion, linker retention, program digest
- FRT-T10: VM stale/peer frames plus JSONL selected-stop-task case
- FRT-T11: VM entry and runtime-error cases
- FRT-T15: VM multi-task snapshot refresh
- FRT-T16: `crates/fpas-debug/tests/forced_return.rs`
- FRT-T17: `crates/fpas-debug/tests/dap_forced_return.rs`
- FRT-T18: `editors/vscode/test/debugger_host/forced_return.ts`
- FRT-T19: focused bytecode/unit/linker/VM/debug suites; workspace otherwise green
- FRT-T20: fixture formatting command above
- FRT-T21: blocked only by the unrelated workspace failure above; every other listed command passes

## Resume instructions

The slice is implemented. Re-run the full workspace command after the Notes
reference-count regression is reconciled; only then may FRT-08 and FRT-T21
move to `complete`/`PASS`. Further control-flow work belongs in a new package
that starts from `consciously-deferred.md` rather than reopening these work
IDs. Do not mark a matrix row `PASS` from inspection alone if behavior changes
again.

## Decision log

- 2026-08-13: selected forced return as the next independently specifiable
  deferred operation.
- 2026-08-13: retained uninitialized descendant synthesis and missing
  parameter/capture creation as consciously deferred because their ownership
  and construction semantics remain undefined.
- 2026-08-13: limited the first slice to the active non-entry callee of the task
  responsible for a non-failure stop.
- 2026-08-13: JSONL mapping lives in `jsonl/server/forced_return.rs` to match
  dictionary/sequence ownership; DAP invalidation areas are `stacks` and
  `variables` only on success.
- 2026-08-13: the one-frame commit mirrors `return_from_call` after structural
  preflight and writes the destination register directly so commit stays
  infallible after validation.
- 2026-08-13: re-audit found post-commit rendering/handle failures and an
  unchanged `.fpascu` version; both were corrected before final verification.
