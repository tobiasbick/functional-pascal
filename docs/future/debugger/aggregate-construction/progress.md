# Progress and resume log

Last updated: 2026-08-14

## Current checkpoint

Implemented on 2026-08-13. Explicit metadata-driven discovery and atomic
complete construction of enum, `Result`, and `Option` variants is live on the
shared VM debugger and the JSONL, DAP, and VS Code surfaces. Remaining
exclusions stay in `consciously-deferred.md`. `DBG-D01` is removed from the
central backlog.

Discovery is read-only. Construction requires an exact field set, evaluates
field expressions once in declaration order under one shared budget, and
commits one complete value. Failures leave the target and inspection generation
unchanged. Standard `setVariable` / `setExpression` constructor assignment is
unchanged.

AGC-T19 remains blocked only because the full workspace gate still contains the
unrelated Notes reference-count failure described below.

The 2026-08-14 implementation review fixed runtime target resolution below
active wrapper payloads and indexed containers, cancellation cleanup after
read-only discovery, and VS Code forwarding of unexpected programmatic fields.
It also split the evaluation entry points and JSONL regressions into focused
modules. No FPAS language behavior changed.

## Status ledger

| ID | Status | Last update | Evidence or blocker |
|---|---|---|---|
| AGC-00 | COMPLETE | 2026-08-13 | Plan package created; link and whitespace verification recorded below |
| AGC-01 | COMPLETE | 2026-08-13 | Shared `mutation/variant/metadata.rs`; existing transition/replacement suites green |
| AGC-02 | COMPLETE | 2026-08-13 | `DebugSession::describe_variant`; uninitialized roots and rejection cases |
| AGC-03 | COMPLETE | 2026-08-13 | Exact-field construction, declaration-order evaluation, atomic commit |
| AGC-04 | COMPLETE | 2026-08-13 | `session/variant.rs`; `VariantUnknown` / `VariantFieldSet`; files remain focused |
| AGC-05 | COMPLETE | 2026-08-13 | JSONL `variant.describe` / `variant.construct` plus capabilities and codes |
| AGC-06 | COMPLETE | 2026-08-13 | DAP `fpas/variantDescribe` / `fpas/variantConstruct`; variables invalidation |
| AGC-07 | COMPLETE | 2026-08-13 | VS Code `functionalPascal.debug.constructVariant` plus Extension Host coverage |
| AGC-08 | COMPLETE | 2026-08-13 | Current docs, closed `DBG-D01`, verification evidence below |

## Review fixes

Recorded 2026-08-14:

- Variant discovery and construction now resolve the live target before reading
  its wrapper metadata. This permits active `Result.value`, active enum fields
  with names shared by variants, and targets below array or dictionary indexes.
- Field expressions after dynamic index selectors share one evaluation budget
  and retain declaration-order evaluation.
- Read-only discovery consumes and clears a pending evaluation cancellation in
  the same way as other debugger evaluation operations.
- The VS Code command preserves unexpected programmatic fields so the shared
  engine can reject them through the exact-field-set contract.
- Evaluation batch entry points and JSONL construction regressions were split
  into focused files to keep the touched modules below the project's size
  threshold.

## Evidence log

Recorded 2026-08-14 after implementation review:

```text
cargo fmt --all -- --check
# exit 0

cargo clippy -p fpas-vm -p fpas-debug --offline -- -D warnings
# exit 0

cargo build --offline
# exit 0

cargo run -p fpas-cli --offline -- fmt --check tests/debugger/fixtures/variant_construction.fpas
# exit 0

cargo test -p fpas-vm --lib variant --offline
# 23 passed (construction, replacement, transition, cancellation cleanup)

cargo test -p fpas-debug --test variant_construction --test dap_variant_construction --offline
# 3 JSONL + 2 DAP passed

cargo test -p fpas-debug --test variant_replacement --test variant_transition --test dap_variant_replacement --test dap_variant_transition --offline
# 11 passed

cargo test --workspace --no-fail-fast --offline
# exit 101; sole failure is repository_references_find_notes_update_in_the_consuming_program
# (23 actual vs 22 expected NotesUpdate references). Unrelated to this package.

npm test --prefix editors/vscode
# exit 0 (includes nested targets and programmatic extra-field rejection)

npm run package --prefix editors/vscode
# exit 0; VSIX includes out/src/debugger/variantConstructionCommand.js

git diff --check
# exit 0 (CRLF conversion warnings only)
```

### 2026-08-13 — AGC-01 through AGC-08

- Status: COMPLETE
- Files changed:
  - `crates/fpas-vm/src/vm/debug/mutation/variant/` (metadata, construct, diagnostics, models)
  - `crates/fpas-vm/src/vm/debug/session/variant.rs`
  - `crates/fpas-vm/src/vm/debug/mutation/transition/suffix.rs` (reuses descriptors)
  - `crates/fpas-debug/src/jsonl/server/variant.rs` and `crates/fpas-debug/src/dap/server/variant.rs`
  - `editors/vscode/src/debugger/variantConstructionCommand.ts`
  - `tests/debugger/fixtures/variant_construction.fpas`
  - VM/JSONL/DAP/Extension Host tests
  - `docs/pascal/tools/debugger.md`, `debugger-jsonl.md`, `debugger-dap.md`, `editor-integration.md`
  - `docs/future/debugger/` backlog and this plan package
- Decisions confirmed: AGC-D01 through AGC-D10 unchanged. No FPAS language change.
- Remaining risk: workspace gate still contains the unrelated NotesUpdate reference-count failure.
- Next action: none for this package. Remaining mutation work is `DBG-D02` / `DBG-D03`.

### 2026-08-14 — implementation review

- Status: COMPLETE; AGC-T19 remains independently blocked.
- Fixes: runtime target type resolution, indexed-target evaluation, discovery
  cancellation cleanup, exact-field forwarding in VS Code, focused module
  splits, and matching positive/negative regressions.
- Verification: targeted VM/JSONL/DAP tests, Rust formatting/build/library
  Clippy, FPAS fixture formatting, full VS Code Extension Host test, and VSIX
  packaging pass. The full workspace test has only the existing Notes reference
  count failure described above.

## Resume checklist

This package is implemented. To resume later work, select a remaining row from
[`../deferred.md`](../deferred.md) rather than reopening `DBG-D01`.
