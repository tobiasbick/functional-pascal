# Progress

Last updated: 2026-08-12

## Current checkpoint

Implementation of variant transition assignment is present and verified on
`codex/fpas-debugger`. Remaining exclusions stay in
`consciously-deferred.md`. No commit belongs to this checkpoint unless the
user requests one.

## Work-package status

| ID | State | Evidence |
|---|---|---|
| VTA-01 | complete | VM/JSONL/DAP negative cases use the frozen error families |
| VTA-02 | complete | debugger-only token adaptation plus parser reserved-word regression |
| VTA-03 | complete | `mutation/transition/` suffix resolution |
| VTA-04 | complete | payload validation plus enum/Result/Option construction |
| VTA-05 | complete | existing `mutation::commit`; success expires handles |
| VTA-06 | complete | JSONL, DAP, selected-task, VS Code host tests |
| VTA-07 | complete | `docs/pascal/tools/` and editor docs describe verified behavior |
| VTA-08 | complete | commands below |

Review fixes keep constructor keywords reserved in FPAS source, resolve the
qualified target before evaluating its replacement, and exercise real inactive
array and dictionary transitions. The VS Code evidence covers the Watch
**Set Value**/DAP surface; Debug Console input remains read-only evaluation.

## Evidence log

```text
cargo fmt --all -- --check
cargo clippy -p fpas-vm -p fpas-debug -p fpas-parser --all-features --locked -- -D warnings
cargo build
cargo test --workspace --no-fail-fast
cargo run -p fpas-cli -- fmt --check tests/debugger/fixtures/variant_transition.fpas
cd editors/vscode && npm test
git diff --check
```

All of the commands above completed with exit code 0 on 2026-08-12.

## Resume instructions

1. Confirm the current branch and preserve unrelated working-tree changes.
2. Read `README.md` and `consciously-deferred.md` before expanding scope.
3. Do not recreate the runtime design from memory; inspect
   `crates/fpas-vm/src/vm/debug/mutation/transition/`.
