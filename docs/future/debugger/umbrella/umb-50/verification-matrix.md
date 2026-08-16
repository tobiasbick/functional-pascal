# UMB-50 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V50-01` | Protocol stdout never contains raw debuggee bytes | Transport/adapter tests | pass |
| `V50-02` | Debuggee input is not parsed as JSONL or DAP | Negative framing tests | pass |
| `V50-03` | Disconnect and EOF clean up host handles without hidden execution | VM/host tests | pass |
| `V50-04` | Captured output remains bounded by the session output limit | Existing output-limit tests plus gaps | pending |
| `V50-05` | Live terminal input is ordered, cancellable, and reports EOF | VM/adapter tests or child-not-started evidence | pass |
| `V50-06` | TUI/graph handlers do not run while the session is stopped | VM stopped-dispatch tests | pending |
| `V50-07` | Pause during an in-progress host intrinsic is cooperative | Existing blocking-intrinsic test plus `UMB-50D` decision | pending |
| `V50-08` | JSONL and DAP expose equivalent I/O events and failures | Paired adapter tests | pass |
| `V50-09` | VS Code exposes only supported debuggee I/O | Extension-host tests | pass |
| `V50-10` | Current docs describe implemented behavior and limitations only | Documentation review | pass |
| `V50-11` | No FPAS syntax, semantic, or language-spec change | Diff review | pending |
| `V50-12` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pending |

## Standard final commands

```powershell
cargo fmt --all -- --check
cargo build --workspace --locked
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings
cargo test --workspace --locked --no-fail-fast
npm test --prefix editors/vscode
git diff --check
```

Add targeted commands to [progress.md](progress.md) as ownership becomes
concrete. If `.fpas` fixtures change, also run their formatter check and the
applicable FPAS test bundle.
