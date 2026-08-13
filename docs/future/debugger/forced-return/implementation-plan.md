# Implementation plan

Stable IDs remain valid across resumed sessions.

## Intended file layout

```text
crates/fpas-bytecode/src/debug/callable.rs
  — MODIFY: optional portable function result type
crates/fpas-bytecode/src/validate/debug_types.rs
  — MODIFY: validate result type references

crates/fpas-compiler/src/bytecode/debug.rs
  — MODIFY: lower IR signature result into debug metadata

crates/fpas-unit/src/object/
  metadata.rs              — MODIFY: serialized object result type
  mod.rs                   — MODIFY: executable/object conversion
  validation.rs            — MODIFY: object-local result type validation

crates/fpas-linker/src/
  debug_types.rs           — MODIFY: retain result-only debug types
  source_map.rs            — MODIFY: translate object result type IDs

crates/fpas-vm/src/vm/debug/
  mod.rs                   — MODIFY: internal forced-return module
  forced_return/
    mod.rs                 — NEW: operation contract and stable errors
    validate.rs            — NEW: return convention, category, and type proof
    commit.rs              — NEW: preflighted one-frame Worker transition
  session/
    forced_return.rs       — NEW: evaluate, commit, refresh, and render result
  tests/
    forced_return/
      mod.rs               — NEW: focused executable and helpers
      cases.rs             — NEW: frame, task, type, limit, and atomicity cases

crates/fpas-debug/src/jsonl/server/forced_return.rs
  — NEW: `frame.return` arguments and response
crates/fpas-debug/src/dap/server/forced_return.rs
  — NEW: `fpas/forceReturn` mapping and invalidation
crates/fpas-debug/tests/
  forced_return.rs         — NEW: JSONL transcript and task ownership
  dap_forced_return.rs     — NEW: DAP parity and invalidation

editors/vscode/src/debugger/forcedReturnCommand.ts
  — NEW: command bound to the active depth-zero frame
editors/vscode/src/extension.ts
editors/vscode/package.json
  — MODIFY: register and contribute the command
editors/vscode/test/debugger_host/forced_return.ts
  — NEW: real Extension Host scenario

tests/debugger/fixtures/forced_return.fpas
  — NEW: nested function/procedure continuation fixture
```

Recheck paths and line counts before implementation. `calls.rs` already owns
normal call/return behavior and must not absorb protocol or debugger policy;
`session/mutation.rs` remains value-assignment-only. Keep forced return in its
own focused debugger modules.

## Work packages

| ID | Work package | Depends on | State | Exit gate |
|---|---|---|---|---|
| FRT-01 | Freeze request shapes, eligible stop/frame/task states, result categories, stable errors, and source fixture | none | complete | Contract tests fail only because the operation is absent |
| FRT-02 | Preserve optional portable result types through compiler, object, linker, `.fpascu`, and `.fpascp` | FRT-01 | complete | Source functions retain translated result IDs; malformed and metadata-less cases are deterministic |
| FRT-03 | Implement bounded result validation and an atomic one-frame runtime transition | FRT-02 | complete | Preflight failure preserves the worker byte-for-byte; success restores exactly one caller |
| FRT-04 | Add session orchestration, selected-task ownership, fresh stop state, rendering, and inspection invalidation | FRT-03 | complete | Caller is immediately inspectable and continuation consumes the forced result |
| FRT-05 | Add JSONL `frame.return` with machine-readable capabilities, codes, hints, and transcripts | FRT-04 | complete | External clients can perform and diagnose the operation without DAP |
| FRT-06 | Add DAP `fpas/forceReturn`, equivalent fields/errors, negotiated stack/variable invalidation, and VS Code command UX | FRT-05 | complete | DAP and JSONL parity plus real Extension Host coverage pass |
| FRT-07 | Update current debugger/editor docs and reconcile deferred scope | FRT-06 | complete | `docs/pascal/tools/` describes only verified behavior and links remaining exclusions |
| FRT-08 | Run focused/full verification and record exact evidence | FRT-07 | blocked | The forced-return slice is green, but the workspace gate still has the unrelated Notes reference-count failure recorded in `progress.md` |

## Dependency order

`FRT-01 -> FRT-02 -> FRT-03 -> FRT-04 -> FRT-05 -> FRT-06 -> FRT-07 -> FRT-08`

## Stop rules

- Stop at FRT-02 if result types cannot remain optional for metadata-less code
  or cannot be translated without display-name inference.
- Stop at FRT-03 if any validation can fail after the call stack or register
  window changes and rollback is not exact.
- Stop at FRT-04 if the operation must run an instruction, scheduler turn, or
  hidden task transition to reach the caller.
- Stop at FRT-05 if JSONL needs adapter-specific semantics instead of one
  protocol-neutral session result.
- Stop at FRT-06 rather than using standard `stepOut`, `setExpression`, or a
  fake Variables child for a non-standard operation.
- Stop before broadening to entry frames, older frames, peer tasks, failed
  stops, Dynamic, function/task/opaque results, or source-language changes.

## Required verification commands

```text
cargo fmt --all -- --check
cargo clippy -p fpas-bytecode -p fpas-compiler -p fpas-unit -p fpas-linker -p fpas-vm -p fpas-debug --all-features --locked -- -D warnings
cargo build --locked
cargo test -p fpas-bytecode --locked
cargo test -p fpas-unit --locked
cargo test -p fpas-linker --locked
cargo test -p fpas-vm --lib forced_return --locked
cargo test -p fpas-debug --test forced_return --locked
cargo test -p fpas-debug --test dap_forced_return --locked
cargo test --workspace --no-fail-fast --locked
cargo run -p fpas-cli --locked -- fmt --check tests/debugger/fixtures/forced_return.fpas
cd editors/vscode && npm test
git diff --check
```
